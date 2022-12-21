use std::{collections::HashMap, fmt::Display, slice::Iter};

use super::inputs::key::Key;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    ClosePopUp,
    Quit,
    ShowInput,
    NavigateUp,
    NavigateDown,
    FilterHttps,
    FilterHttp,
    FilterRsync,
    FilterSyncing,
}

impl Action {
    pub fn iterator() -> Iter<'static, Action> {
        static ACTIONS: [Action; 9] = [
            Action::Quit,
            Action::ClosePopUp,
            Action::ShowInput,
            Action::NavigateUp,
            Action::NavigateDown,
            Action::FilterHttp,
            Action::FilterHttps,
            Action::FilterRsync,
            Action::FilterSyncing,
        ];
        ACTIONS.iter()
    }

    pub fn keys(&self) -> &[Key] {
        match self {
            Action::Quit => &[Key::Ctrl('c'), Key::Char('q')],
            Action::ClosePopUp => &[Key::Ctrl('p')],
            Action::ShowInput => &[Key::Ctrl('f'), Key::Char('/')],
            Action::NavigateUp => &[Key::Char('k'), Key::Up],
            Action::NavigateDown => &[Key::Char('j'), Key::Down],
            Action::FilterHttps => &[Key::Ctrl('s')],
            Action::FilterHttp => &[Key::Ctrl('h')],
            Action::FilterRsync => &[Key::Ctrl('r')],
            Action::FilterSyncing => &[Key::Ctrl('o')],
        }
    }
}

impl Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Action::ClosePopUp => "close popup",
            Action::Quit => "quit",
            Action::ShowInput => "toggle filter",
            Action::NavigateUp => "up",
            Action::NavigateDown => "down",
            Action::FilterHttps => "toggle https",
            Action::FilterHttp => "toggle http",
            Action::FilterRsync => "toggle rsync",
            Action::FilterSyncing => "toggle in-sync",
        };
        write!(f, "{str}")
    }
}

/// The application should have some contextual actions.
#[derive(Default, Debug, Clone)]
pub struct Actions(Vec<Action>);

impl Actions {
    /// Given a key, find the corresponding action
    pub fn find(&self, key: Key) -> Option<&Action> {
        Action::iterator()
            .filter(|action| self.0.contains(action))
            .find(|action| action.keys().contains(&key))
    }

    /// Get contextual actions.
    /// (just for building a help view)
    pub fn actions(&self) -> &[Action] {
        self.0.as_slice()
    }
}

impl From<Vec<Action>> for Actions {
    /// Build contextual action
    ///
    /// # Panics
    ///
    /// If two actions have same key
    fn from(actions: Vec<Action>) -> Self {
        // Check key unicity
        let mut map: HashMap<Key, Vec<Action>> = HashMap::new();
        for action in actions.iter() {
            for key in action.keys().iter() {
                match map.get_mut(key) {
                    Some(vec) => vec.push(*action),
                    None => {
                        map.insert(*key, vec![*action]);
                    }
                }
            }
        }
        let errors = map
            .iter()
            .filter(|(_, actions)| actions.len() > 1) // at least two actions share same shortcut
            .map(|(key, actions)| {
                let actions = actions
                    .iter()
                    .map(Action::to_string)
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("Conflict key {key} with actions {actions}")
            })
            .collect::<Vec<_>>();
        if !errors.is_empty() {
            panic!("{}", errors.join("; "))
        }

        // Ok, we can create contextual actions
        Self(actions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_find_action_by_key() {
        let actions: Actions = vec![Action::Quit, Action::ClosePopUp].into();
        let result = actions.find(Key::Ctrl('c'));
        assert_eq!(result, Some(&Action::Quit));
    }

    #[test]
    fn should_find_action_by_key_not_found() {
        let actions: Actions = vec![Action::Quit, Action::ClosePopUp].into();
        let result = actions.find(Key::Alt('w'));
        assert_eq!(result, None);
    }

    #[test]
    fn should_create_actions_from_vec() {
        let _actions: Actions = vec![Action::Quit, Action::ClosePopUp].into();
    }

    #[test]
    #[should_panic]
    fn should_panic_when_create_actions_conflict_key() {
        let _actions: Actions = vec![Action::Quit, Action::ClosePopUp, Action::Quit].into();
    }
}
