use crate::screens::contacts::Contacts;
use crate::screens::personal_settings::PersonalSettings;
use crate::screens::sign_in::SignIn;

pub enum Screen {
    SignIn(SignIn),
    Contacts(Contacts),
    PersonalSettings(PersonalSettings),
}
