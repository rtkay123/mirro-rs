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
    FilterFtp,
    FilterRsync,
    FilterSyncing,
    FilterIpv4,
    FilterIpv6,
    FilterIsos,
    ViewSortAlphabetically,
    ViewSortMirrorCount,
    ToggleSelect,
    SelectionSortCompletionPct,
    SelectionSortDelay,
    SelectionSortDuration,
    SelectionSortScore,
    Export,
}

impl Action {
    pub fn iterator() -> Iter<'static, Action> {
        static ACTIONS: [Action; 21] = [
            Action::Quit,
            Action::ClosePopUp,
            Action::ShowInput,
            Action::NavigateUp,
            Action::NavigateDown,
            Action::FilterHttp,
            Action::FilterHttps,
            Action::FilterRsync,
            Action::FilterFtp,
            Action::FilterIpv4,
            Action::FilterIpv6,
            Action::FilterIsos,
            Action::FilterSyncing,
            Action::ViewSortMirrorCount,
            Action::ViewSortAlphabetically,
            Action::ToggleSelect,
            Action::SelectionSortCompletionPct,
            Action::SelectionSortDelay,
            Action::SelectionSortDuration,
            Action::SelectionSortScore,
            Action::Export,
        ];
        ACTIONS.iter()
    }

    pub fn keys(&self) -> &[Key] {
        match self {
            Action::Quit => &[Key::Ctrl('c'), Key::Char('q')],
            Action::ClosePopUp => &[Key::Ctrl('p')],
            Action::ShowInput => &[Key::Ctrl('i'), Key::Char('/')],
            Action::NavigateUp => &[Key::Char('k'), Key::Up],
            Action::NavigateDown => &[Key::Char('j'), Key::Down],
            Action::FilterHttps => &[Key::Ctrl('s')],
            Action::FilterHttp => &[Key::Ctrl('h')],
            Action::FilterRsync => &[Key::Ctrl('r')],
            Action::FilterFtp => &[Key::Ctrl('f')],
            Action::FilterSyncing => &[Key::Ctrl('o')],
            Action::ViewSortAlphabetically => &[Key::Char('1')],
            Action::ViewSortMirrorCount => &[Key::Char('2')],
            Action::ToggleSelect => &[Key::Char(' ')],
            Action::SelectionSortCompletionPct => &[Key::Char('5')],
            Action::SelectionSortDelay => &[Key::Char('6')],
            Action::SelectionSortDuration => &[Key::Char('7')],
            Action::SelectionSortScore => &[Key::Char('8')],
            Action::Export => &[Key::Ctrl('e')],
            Action::FilterIpv4 => &[Key::Ctrl('4')],
            Action::FilterIpv6 => &[Key::Ctrl('6')],
            Action::FilterIsos => &[Key::Ctrl('5')],
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
            Action::FilterFtp => "toggle ftp",
            Action::FilterSyncing => "toggle in-sync",
            Action::ViewSortAlphabetically => "sort [country] A-Z",
            Action::ViewSortMirrorCount => "sort [country] mirrors",
            Action::ToggleSelect => "[de]select mirror",
            Action::SelectionSortCompletionPct => "sort [selection] completion",
            Action::SelectionSortDelay => "sort [selection] delay",
            Action::SelectionSortDuration => "sort [selection] duration",
            Action::SelectionSortScore => "sort [selection] score",
            Action::Export => "export mirrors",
            Action::FilterIpv4 => "toggle ipv4",
            Action::FilterIpv6 => "toggle ipv6",
            Action::FilterIsos => "toggle isos",
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
            let err = errors.join("; ");
            panic!("{err}")
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
