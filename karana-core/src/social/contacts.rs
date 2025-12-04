//! Contact Management for Kāraṇa OS AR Glasses
//!
//! Store and manage contacts with AR-specific features.

use std::collections::HashMap;
use std::time::Instant;

/// Contact source
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ContactSource {
    /// Local device
    Local,
    /// Google account
    Google,
    /// Apple iCloud
    Apple,
    /// Microsoft account
    Microsoft,
    /// Social media
    Social,
    /// Work/Exchange
    Work,
    /// Other
    Other,
}

/// Contact relationship type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Relationship {
    /// Family member
    Family,
    /// Friend
    Friend,
    /// Colleague
    Colleague,
    /// Acquaintance
    Acquaintance,
    /// Other
    Other,
}

/// Phone number type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PhoneType {
    Mobile,
    Home,
    Work,
    Other,
}

/// Phone number entry
#[derive(Debug, Clone)]
pub struct PhoneNumber {
    /// Number
    pub number: String,
    /// Type
    pub phone_type: PhoneType,
    /// Is primary
    pub primary: bool,
}

/// Email entry
#[derive(Debug, Clone)]
pub struct EmailAddress {
    /// Email address
    pub email: String,
    /// Label (home, work, etc.)
    pub label: String,
    /// Is primary
    pub primary: bool,
}

/// Contact
#[derive(Debug, Clone)]
pub struct Contact {
    /// Unique ID
    pub id: String,
    /// Display name
    pub name: String,
    /// First name
    pub first_name: Option<String>,
    /// Last name
    pub last_name: Option<String>,
    /// Nickname
    pub nickname: Option<String>,
    /// Phone numbers
    pub phones: Vec<PhoneNumber>,
    /// Email addresses
    pub emails: Vec<EmailAddress>,
    /// Photo URL
    pub photo_url: Option<String>,
    /// Source
    pub source: ContactSource,
    /// Relationship
    pub relationship: Option<Relationship>,
    /// Is favorite
    pub favorite: bool,
    /// Notes
    pub notes: Option<String>,
    /// Company
    pub company: Option<String>,
    /// Job title
    pub job_title: Option<String>,
    /// Last contact time
    pub last_contact: Option<Instant>,
    /// Contact count
    pub contact_count: u32,
    /// Custom AR display preferences
    pub ar_preferences: ContactARPreferences,
}

/// AR display preferences for contact
#[derive(Debug, Clone)]
pub struct ContactARPreferences {
    /// Show floating label when nearby
    pub show_label: bool,
    /// Custom AR color
    pub ar_color: Option<(f32, f32, f32)>,
    /// Custom notification sound
    pub notification_sound: Option<String>,
    /// Priority level (affects notification position)
    pub priority: ContactPriority,
}

impl Default for ContactARPreferences {
    fn default() -> Self {
        Self {
            show_label: false,
            ar_color: None,
            notification_sound: None,
            priority: ContactPriority::Normal,
        }
    }
}

/// Contact priority level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ContactPriority {
    /// VIP - always show immediately
    VIP,
    /// High priority
    High,
    /// Normal
    Normal,
    /// Low priority
    Low,
}

impl Contact {
    /// Create new contact
    pub fn new(id: String, name: String) -> Self {
        Self {
            id,
            name,
            first_name: None,
            last_name: None,
            nickname: None,
            phones: Vec::new(),
            emails: Vec::new(),
            photo_url: None,
            source: ContactSource::Local,
            relationship: None,
            favorite: false,
            notes: None,
            company: None,
            job_title: None,
            last_contact: None,
            contact_count: 0,
            ar_preferences: ContactARPreferences::default(),
        }
    }
    
    /// Get display name
    pub fn display_name(&self) -> &str {
        if let Some(ref nick) = self.nickname {
            nick
        } else {
            &self.name
        }
    }
    
    /// Get primary phone
    pub fn primary_phone(&self) -> Option<&PhoneNumber> {
        self.phones.iter().find(|p| p.primary).or_else(|| self.phones.first())
    }
    
    /// Get primary email
    pub fn primary_email(&self) -> Option<&EmailAddress> {
        self.emails.iter().find(|e| e.primary).or_else(|| self.emails.first())
    }
    
    /// Add phone number
    pub fn add_phone(&mut self, number: String, phone_type: PhoneType, primary: bool) {
        if primary {
            // Clear other primaries
            for phone in &mut self.phones {
                phone.primary = false;
            }
        }
        self.phones.push(PhoneNumber {
            number,
            phone_type,
            primary,
        });
    }
    
    /// Add email
    pub fn add_email(&mut self, email: String, label: String, primary: bool) {
        if primary {
            for e in &mut self.emails {
                e.primary = false;
            }
        }
        self.emails.push(EmailAddress {
            email,
            label,
            primary,
        });
    }
    
    /// Record contact interaction
    pub fn record_interaction(&mut self) {
        self.last_contact = Some(Instant::now());
        self.contact_count += 1;
    }
    
    /// Is VIP
    pub fn is_vip(&self) -> bool {
        self.ar_preferences.priority == ContactPriority::VIP || self.favorite
    }
}

/// Contact group
#[derive(Debug, Clone)]
pub struct ContactGroup {
    /// Group ID
    pub id: String,
    /// Group name
    pub name: String,
    /// Contact IDs in this group
    pub contact_ids: Vec<String>,
    /// Group color
    pub color: Option<(f32, f32, f32)>,
}

impl ContactGroup {
    /// Create new group
    pub fn new(id: String, name: String) -> Self {
        Self {
            id,
            name,
            contact_ids: Vec::new(),
            color: None,
        }
    }
    
    /// Add contact to group
    pub fn add_contact(&mut self, contact_id: String) {
        if !self.contact_ids.contains(&contact_id) {
            self.contact_ids.push(contact_id);
        }
    }
    
    /// Remove contact from group
    pub fn remove_contact(&mut self, contact_id: &str) {
        self.contact_ids.retain(|id| id != contact_id);
    }
    
    /// Get contact count
    pub fn count(&self) -> usize {
        self.contact_ids.len()
    }
}

/// Contact manager
#[derive(Debug)]
pub struct ContactManager {
    /// All contacts
    contacts: HashMap<String, Contact>,
    /// Groups
    groups: HashMap<String, ContactGroup>,
    /// Favorites list (ordered)
    favorites: Vec<String>,
    /// Recent contacts
    recent: Vec<String>,
    /// Max recent
    max_recent: usize,
}

impl ContactManager {
    /// Create new contact manager
    pub fn new() -> Self {
        Self {
            contacts: HashMap::new(),
            groups: HashMap::new(),
            favorites: Vec::new(),
            recent: Vec::new(),
            max_recent: 20,
        }
    }
    
    /// Add contact
    pub fn add_contact(&mut self, contact: Contact) {
        let id = contact.id.clone();
        let is_favorite = contact.favorite;
        self.contacts.insert(id.clone(), contact);
        
        if is_favorite && !self.favorites.contains(&id) {
            self.favorites.push(id);
        }
    }
    
    /// Get contact by ID
    pub fn get(&self, id: &str) -> Option<&Contact> {
        self.contacts.get(id)
    }
    
    /// Get mutable contact
    pub fn get_mut(&mut self, id: &str) -> Option<&mut Contact> {
        self.contacts.get_mut(id)
    }
    
    /// Remove contact
    pub fn remove(&mut self, id: &str) -> Option<Contact> {
        self.favorites.retain(|f| f != id);
        self.recent.retain(|r| r != id);
        self.contacts.remove(id)
    }
    
    /// Search contacts by name
    pub fn search(&self, query: &str) -> Vec<&Contact> {
        let query_lower = query.to_lowercase();
        self.contacts.values()
            .filter(|c| {
                c.name.to_lowercase().contains(&query_lower) ||
                c.nickname.as_ref().map(|n| n.to_lowercase().contains(&query_lower)).unwrap_or(false)
            })
            .collect()
    }
    
    /// Search by phone number
    pub fn search_by_phone(&self, number: &str) -> Option<&Contact> {
        let normalized = number.chars().filter(|c| c.is_digit(10)).collect::<String>();
        self.contacts.values()
            .find(|c| c.phones.iter().any(|p| {
                let phone_normalized = p.number.chars().filter(|c| c.is_digit(10)).collect::<String>();
                phone_normalized.ends_with(&normalized) || normalized.ends_with(&phone_normalized)
            }))
    }
    
    /// Get favorites
    pub fn favorites(&self) -> Vec<&Contact> {
        self.favorites.iter()
            .filter_map(|id| self.contacts.get(id))
            .collect()
    }
    
    /// Toggle favorite
    pub fn toggle_favorite(&mut self, id: &str) {
        if let Some(contact) = self.contacts.get_mut(id) {
            contact.favorite = !contact.favorite;
            
            if contact.favorite {
                if !self.favorites.contains(&id.to_string()) {
                    self.favorites.push(id.to_string());
                }
            } else {
                self.favorites.retain(|f| f != id);
            }
        }
    }
    
    /// Add to recent
    pub fn add_recent(&mut self, id: &str) {
        self.recent.retain(|r| r != id);
        if self.recent.len() >= self.max_recent {
            self.recent.remove(0);
        }
        self.recent.push(id.to_string());
        
        // Update contact interaction
        if let Some(contact) = self.contacts.get_mut(id) {
            contact.record_interaction();
        }
    }
    
    /// Get recent contacts
    pub fn recent(&self) -> Vec<&Contact> {
        self.recent.iter()
            .rev()
            .filter_map(|id| self.contacts.get(id))
            .collect()
    }
    
    /// Get VIP contacts
    pub fn vips(&self) -> Vec<&Contact> {
        self.contacts.values()
            .filter(|c| c.is_vip())
            .collect()
    }
    
    /// Create group
    pub fn create_group(&mut self, id: String, name: String) {
        self.groups.insert(id.clone(), ContactGroup::new(id, name));
    }
    
    /// Get group
    pub fn get_group(&self, id: &str) -> Option<&ContactGroup> {
        self.groups.get(id)
    }
    
    /// Get mutable group
    pub fn get_group_mut(&mut self, id: &str) -> Option<&mut ContactGroup> {
        self.groups.get_mut(id)
    }
    
    /// Get contacts in group
    pub fn contacts_in_group(&self, group_id: &str) -> Vec<&Contact> {
        self.groups.get(group_id)
            .map(|g| {
                g.contact_ids.iter()
                    .filter_map(|id| self.contacts.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }
    
    /// Total contact count
    pub fn count(&self) -> usize {
        self.contacts.len()
    }
    
    /// Get all contacts
    pub fn all(&self) -> Vec<&Contact> {
        self.contacts.values().collect()
    }
    
    /// Get most contacted person
    pub fn most_contacted(&self) -> Option<String> {
        self.contacts.values()
            .max_by_key(|c| c.contact_count)
            .filter(|c| c.contact_count > 0)
            .map(|c| c.name.clone())
    }
}

impl Default for ContactManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_contact_creation() {
        let contact = Contact::new("1".to_string(), "John Doe".to_string());
        assert_eq!(contact.name, "John Doe");
    }
    
    #[test]
    fn test_add_phone() {
        let mut contact = Contact::new("1".to_string(), "John".to_string());
        contact.add_phone("555-1234".to_string(), PhoneType::Mobile, true);
        
        assert_eq!(contact.phones.len(), 1);
        assert!(contact.primary_phone().is_some());
    }
    
    #[test]
    fn test_contact_manager() {
        let mut manager = ContactManager::new();
        
        let contact = Contact::new("1".to_string(), "John Doe".to_string());
        manager.add_contact(contact);
        
        assert_eq!(manager.count(), 1);
        assert!(manager.get("1").is_some());
    }
    
    #[test]
    fn test_search() {
        let mut manager = ContactManager::new();
        
        manager.add_contact(Contact::new("1".to_string(), "John Doe".to_string()));
        manager.add_contact(Contact::new("2".to_string(), "Jane Smith".to_string()));
        
        let results = manager.search("john");
        assert_eq!(results.len(), 1);
    }
    
    #[test]
    fn test_favorites() {
        let mut manager = ContactManager::new();
        
        let mut contact = Contact::new("1".to_string(), "John".to_string());
        contact.favorite = true;
        manager.add_contact(contact);
        
        assert_eq!(manager.favorites().len(), 1);
        
        manager.toggle_favorite("1");
        assert_eq!(manager.favorites().len(), 0);
    }
    
    #[test]
    fn test_groups() {
        let mut manager = ContactManager::new();
        
        manager.add_contact(Contact::new("1".to_string(), "John".to_string()));
        manager.create_group("g1".to_string(), "Family".to_string());
        
        if let Some(group) = manager.get_group_mut("g1") {
            group.add_contact("1".to_string());
        }
        
        let contacts = manager.contacts_in_group("g1");
        assert_eq!(contacts.len(), 1);
    }
    
    #[test]
    fn test_recent() {
        let mut manager = ContactManager::new();
        
        manager.add_contact(Contact::new("1".to_string(), "John".to_string()));
        manager.add_recent("1");
        
        let recent = manager.recent();
        assert_eq!(recent.len(), 1);
    }
}
