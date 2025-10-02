use crate::models::contact::Contact;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Clone, Default)]
pub struct ContactRepository {
    contacts: Arc<Mutex<HashMap<Arc<String>, Contact>>>,
}

impl ContactRepository {
    pub fn new() -> Self {
        Self {
            contacts: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn get_contact(&self, email: &String) -> Option<Contact> {
        if let Ok(contacts) = self.contacts.lock() {
            contacts.get(email).cloned()
        } else {
            None
        }
    }

    pub fn add_contacts(&self, contacts: &[Contact]) {
        if let Ok(mut contacts_lock) = self.contacts.lock() {
            contacts_lock.reserve(contacts.len());
            for contact in contacts {
                contacts_lock.insert(contact.email.clone(), contact.clone());
            }
        }
    }

    pub fn update_contacts(&self, contacts: &[Contact]) {
        if let Ok(mut contacts_lock) = self.contacts.lock() {
            for contact in contacts {
                contacts_lock.insert(contact.email.clone(), contact.clone());
            }
        }
    }

    pub fn remove_contact(&self, email: &String) {
        if let Ok(mut contacts_lock) = self.contacts.lock() {
            contacts_lock.remove(email);
        }
    }
}
