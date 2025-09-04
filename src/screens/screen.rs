use crate::screens::add_contact::AddContact;
use crate::screens::contacts::contacts::Contacts;
use crate::screens::conversation::conversation::Conversation;
use crate::screens::dialog::Dialog;
use crate::screens::personal_settings::PersonalSettings;
use crate::screens::sign_in::sign_in::SignIn;

pub enum Screen {
    SignIn(SignIn),
    Contacts(Contacts),
    PersonalSettings(PersonalSettings),
    Conversation(Conversation),
    Dialog(Dialog),
    AddContact(AddContact),
}
