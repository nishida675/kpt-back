mod controllers {
    mod accounts;
    mod root;
    pub mod boards;

    pub use accounts::accounts;
    pub use boards::boards;
    pub use root::app;
}

mod database;

mod entities {
    mod account;
    mod board;
    mod ticket;

    pub use account::Account;
    pub use board::Board;
    pub use ticket::Ticket;
}

mod repos_impl {
    mod accounts;
    mod boards;
    mod tickets;

    pub use accounts::AccountsImpl;
    pub use boards::BoardsImpl;
    pub use tickets::TicketsImpl;
}

pub mod repositories;


mod services {
    mod accounts;
    mod boards;
    mod tickets;

    pub use accounts::{create_account, create_session, SessionToken};
    pub use boards::{get_all_boards, get_board_by_id, save_board, update_board, delete_board};
    pub use tickets::{
        get_all_tickets, save_ticket, update_ticket, delete_ticket,
    };
}

mod request;

pub use controllers::app;

mod constants {
    pub const AXUM_SESSION_COOKIE_NAME: &str = "rustwi_session";
    pub const AXUM_SESSION_USER_ID_KEY: &str = "uid";
    pub const ENV_KEY_DATABASE_URL: &str = "DATABASE_URL";
}