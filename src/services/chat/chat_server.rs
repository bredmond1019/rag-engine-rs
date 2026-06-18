use crate::models::Article;
use crate::services::ai::AIService;
use crate::services::EmbeddingService;
use crate::{db::DbPool, models::message::Message};

use actix::prelude::*;
use futures::StreamExt;
use log::{debug, error, info};
use pgvector::Vector;
use serde::Deserialize;
use std::{collections::HashMap, sync::Arc};

use super::chat_session::SessionId;

#[derive(Message)]
#[rtype(result = "()")]
pub struct Connect {
    pub addr: Recipient<Message>,
    pub id: SessionId,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: SessionId,
}

#[derive(Message, Deserialize)]
#[rtype(result = "()")]
pub struct ClientMessage {
    pub session_id: SessionId,
    pub message: String,
}

pub struct ChatServer {
    sessions: HashMap<SessionId, Recipient<Message>>,
    ai_service: AIService,
    db_pool: Arc<DbPool>,
}

impl ChatServer {
    pub fn new(db_pool: Arc<DbPool>) -> Self {
        Self {
            sessions: HashMap::new(),
            ai_service: AIService::new(),
            db_pool,
        }
    }

    /// Build a retrieval-augmented prompt for `text`: embed the query, find the
    /// most relevant articles, and fold them into a grounded prompt. The caller
    /// streams the LLM response from this prompt, so retrieval happens off the
    /// actor thread (associated fn — takes the pool by `Arc`, not `&self`).
    async fn build_rag_prompt(
        db_pool: Arc<DbPool>,
        text: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let embedding_service = EmbeddingService::new();
        let query_embedding = embedding_service.generate_embedding(text).await?;
        let query_embedding = Vector::from(query_embedding);
        let mut conn = db_pool.get()?;
        let relevant_articles =
            Article::find_relevant_articles(&query_embedding, &mut conn).await?;

        let context = relevant_articles
            .iter()
            .map(|(article, similarity)| {
                format!(
                    "Article: {} (Similarity: {:.2})\nContent: {}",
                    article.title,
                    similarity,
                    article
                        .markdown_content
                        .as_deref()
                        .unwrap_or(&article.title)
                )
            })
            .collect::<Vec<String>>()
            .join("\n\n");

        Ok(format!(
            "Based on the following context and the user's question, provide a helpful answer. Include references to the relevant articles.\n\nContext:\n{}\n\nUser Question: {}\n\nAnswer:",
            context, text
        ))
    }
}

impl Actor for ChatServer {
    type Context = Context<Self>;
}

impl Handler<Connect> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) {
        info!("ChatSession connected: {:?}", msg.id);
        self.sessions.insert(msg.id, msg.addr);
        info!("Total active sessions: {}", self.sessions.len());
    }
}

impl Handler<Disconnect> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        info!("ChatSession disconnected: {:?}", msg.id);
        self.sessions.remove(&msg.id);
        info!("Total active sessions: {}", self.sessions.len());
    }
}

impl Handler<ClientMessage> for ChatServer {
    type Result = ResponseFuture<()>;

    fn handle(&mut self, client_message: ClientMessage, _: &mut Context<Self>) -> Self::Result {
        debug!(
            "Received message from session {:?}: {}",
            client_message.session_id, client_message.message
        );
        let mut ai_service = self.ai_service.clone();
        let sessions = self.sessions.clone();
        let db_pool = self.db_pool.clone();
        let id = client_message.session_id;
        let user_message = client_message.message;

        Box::pin(async move {
            // Retrieve relevant articles and build a grounded (RAG) prompt before
            // streaming, so chat answers are anchored in the help docs.
            let prompt = match ChatServer::build_rag_prompt(db_pool, &user_message).await {
                Ok(prompt) => prompt,
                Err(e) => {
                    error!("Failed to build RAG prompt for session {:?}: {}", id, e);
                    if let Some(addr) = sessions.get(&id) {
                        addr.do_send(Message::new(
                            "Sorry, I couldn't process your request. Please try again.".to_string(),
                            true,
                        ));
                    }
                    return;
                }
            };

            info!("Generating AI response for session {:?}", id);
            match ai_service.generate_stream_response(prompt).await {
                Ok(stream) => {
                    info!("AI response stream generated for session {:?}", id);
                    let addr = sessions.get(&id).cloned();
                    if let Some(addr) = addr {
                        tokio::spawn(async move {
                            let mut stream = stream;
                            while let Some(chunk_result) = stream.next().await {
                                match chunk_result {
                                    Ok(chunk) => {
                                        if !chunk.is_empty() {
                                            let ai_message = Message::new(chunk, false);
                                            addr.do_send(ai_message);
                                        }
                                    }
                                    Err(e) => {
                                        error!("Error in AI response stream: {}", e);
                                        let error_message = Message::new(
                                            "Sorry, there was an error processing your request."
                                                .to_string(),
                                            true,
                                        );
                                        addr.do_send(error_message);
                                        break;
                                    }
                                }
                            }
                            // Send end of stream message
                            let end_message = Message::new("".to_string(), true);
                            addr.do_send(end_message);
                        });
                    } else {
                        error!("Session {:?} not found", id);
                    }
                }
                Err(e) => {
                    error!("Failed to generate AI response for session {:?}: {}", id, e);
                    if let Some(addr) = sessions.get(&id) {
                        let error_message = Message::new(
                            "Sorry, I couldn't process your request. Please try again.".to_string(),
                            true,
                        );
                        info!("Sending error message to session {:?}", id);
                        addr.do_send(error_message);
                    }
                }
            }
        })
    }
}
