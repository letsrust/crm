use tonic::Status;
use tracing::{info, warn};

use crate::{
    pb::{send_request::Msg, EmailMessage, SendRequest, SendResponse},
    NotificationService,
};

use super::{to_ts, Sender};

impl Sender for EmailMessage {
    async fn send(self, svc: NotificationService) -> Result<SendResponse, Status> {
        let message_id = self.message_id.clone();
        let sender = self.sender.clone();
        let recipients = self.recipients.join(",");
        svc.sender.send(Msg::Email(self)).await.map_err(|e| {
            warn!("Failed to send email message: {:?}", e);
            Status::internal("Failed to send email message")
        })?;

        info!("email sender: {:?}, recipients: {}", sender, recipients);

        Ok(SendResponse {
            message_id,
            timestamp: Some(to_ts()),
        })
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

#[cfg(feature = "test_utils")]
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
