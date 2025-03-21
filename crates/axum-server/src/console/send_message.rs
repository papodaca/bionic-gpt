use crate::authentication::Authentication;
use crate::errors::CustomError;
use axum::{
    extract::{Extension, Form, Path},
    response::IntoResponse,
};
use db::queries::{chats, conversations};
use db::{Conversation, Pool};
use serde::Deserialize;
use validator::Validate;

#[derive(Deserialize, Validate, Default, Debug)]
pub struct Message {
    pub message: String,
    pub prompt_id: i32,
}

pub async fn send_message(
    current_user: Authentication,
    Extension(pool): Extension<Pool>,
    Path(team_id): Path<i32>,
    Form(message): Form<Message>,
) -> Result<impl IntoResponse, CustomError> {
    if message.validate().is_ok() {
        let mut client = pool.get().await?;
        let transaction = client.transaction().await?;

        super::super::rls::set_row_level_security_user(&transaction, &current_user).await?;

        // Get the latest conversation
        let conversation: Result<Conversation, db::TokioPostgresError> =
            conversations::get_latest_conversation()
                .bind(&transaction)
                .one()
                .await;

        let conv_id = if let Ok(conversation) = conversation {
            conversation.id
        } else {
            conversations::create_conversation()
                .bind(&transaction, &team_id)
                .one()
                .await?
        };

        // Store the prompt, ready for the front end webcomponent to pickup
        chats::new_chat()
            .bind(
                &transaction,
                &conv_id,
                &message.prompt_id,
                &message.message,
                &"",
            )
            .await?;

        transaction.commit().await?;

        crate::layout::redirect(&ui_components::routes::console::index_route(team_id))
    } else {
        crate::layout::redirect(&ui_components::routes::console::index_route(team_id))
    }
}
