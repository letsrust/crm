use tonic::Status;
use tracing::warn;

use crate::{
    pb::{send_request::Msg, EmailMessage, SendRequest, SendResponse},
    NotificationService,
};

use super::{to_ts, Sender};

impl Sender for EmailMessage {
    async fn send(self, svc: NotificationService) -> Result<SendResponse, Status> {
        let message_id = self.message_id.clone();
        svc.sender.send(Msg::Email(self)).await.map_err(|e| {
            warn!("Failed to send email message: {:?}", e);
            Status::internal("Failed to send email message")
        })?;

        Ok(SendResponse {
            message_id,
            timestamp: Some(to_ts()),
        })
    }
}

#[cfg(test)]
impl EmailMessage {
    pub fn fake() -> Self {
        use fake::faker::internet::en::SafeEmail;
        use fake::Fake;
        use uuid::Uuid;
        EmailMessage {
            message_id: Uuid::new_v4().to_string(),
            sender: SafeEmail().fake(),
            recipients: vec![SafeEmail().fake()],
            subject: "Hello".to_string(),
            body: "Hello, world!".to_string(),
        }
    }
}

impl From<EmailMessage> for Msg {
    fn from(email: EmailMessage) -> Self {
        Msg::Email(email)
    }
}

impl From<EmailMessage> for SendRequest {
    fn from(email: EmailMessage) -> Self {
        let msg: Msg = email.into();
        SendRequest { msg: Some(msg) }
    }
}