use crate::models::display_picture::DisplayPicture;

pub struct User {
    pub personal_message: Option<String>,
    pub display_picture: Option<DisplayPicture>,
}
