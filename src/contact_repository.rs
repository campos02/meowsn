use crate::models::contact::Contact;
use msnp11_sdk::MsnpList;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Clone, Default)]
pub struct ContactRepository {
    contacts: Arc<RwLock<HashMap<Arc<String>, Contact>>>,
}

impl ContactRepository {
    pub fn new() -> Self {
        Self {
            contacts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn get_contact(&self, email: &String) -> Option<Contact> {
        if let Ok(contacts) = self.contacts.read() {
            contacts.get(email).cloned()
        } else {
            None
        }
    }

    pub fn get_contacts_in_list(&self, list: MsnpList) -> Option<Vec<Contact>> {
        if let Ok(contacts) = self.contacts.read() {
            Some(
                contacts
                    .values()
                    .filter(|contact| contact.lists.contains(&list))
                    .cloned()
                    .collect(),
            )
        } else {
            None
        }
    }

    pub fn add_contacts(&self, contacts: &[Contact]) {
        if let Ok(mut contacts_lock) = self.contacts.write() {
            contacts_lock.reserve(contacts.len());
            for contact in contacts {
                contacts_lock.insert(contact.email.clone(), contact.clone());
            }
        }
    }

    pub fn update_contacts(&self, contacts: &[Contact]) {
        if let Ok(mut contacts_lock) = self.contacts.write() {
            for contact in contacts {
                contacts_lock.insert(contact.email.clone(), contact.clone());
            }
        }
    }

    pub fn remove_contact(&self, email: &String) {
        if let Ok(mut contacts_lock) = self.contacts.write() {
            contacts_lock.remove(email);
        }
    }
}
