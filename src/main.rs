use axum::{
    Router,
    extract::State,
    extract::Json,
    routing::get,
    http::StatusCode,
};
use serde::Serialize;
use std::sync::{Mutex, Arc};
use dotenv::dotenv;

use back_end_paper_2::api::auth::user;

#[derive(Clone)]
struct AppState {
    user_list: Arc<Mutex<user::UserList>>,
}

#[derive(Serialize)]
struct UserListResponse {
    users: Vec<user::User>
}

// async fn add_user (
//     State(state): State<AppState>,
//     Json(body): Json<User>,
// ) -> StatusCode {
//     StatusCode::CREATED
// }

async fn get_all_users (
    State(state): State<AppState>
) -> (StatusCode, Json<UserListResponse>) {
    let users = state.user_list.lock().unwrap();
    let response = UserListResponse {
        users: users.get_all()
    };
    (StatusCode::OK, Json(response))
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let mut new_user_list = user::UserList::new();
    new_user_list.add(user::get_default_admin_user());

    let state = AppState {
        user_list: Arc::new(Mutex::new(new_user_list))
    };

    let app = Router::new()
        .route("/users", get(get_all_users))
        .with_state(state);

    // run it with hyper on localhost:3000
    axum::Server::bind(&"127.0.0.1:8080".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
