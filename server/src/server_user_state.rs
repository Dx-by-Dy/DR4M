use common::UserSID;

use crate::Tx;

pub struct ServerUserState {
    pub usid: Option<UserSID>,
    pub tx: Option<Tx>,
}
