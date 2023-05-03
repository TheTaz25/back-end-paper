use axum::{
  Router,
  extract::{State,Json,Path},
  routing::{get,post},
  http::StatusCode
};
use serde::{Serialize,Deserialize};

use crate::state::AppState;

#[derive(Clone, Serialize)]
pub struct User {
  username: String,
  password: String,
  admin: bool,
}

fn hash_password(password: String) -> Result<String, &'static str> {
  let hash_result = bcrypt::hash(password, 10);
  match hash_result.ok().as_ref() {
    Some(result) => Ok(result.clone()),
    None => Err("Could not generate hash from password")
  }
}

pub fn get_default_admin_user () -> User {
  let admin_username = std::env::var("ADMIN_USER").expect("ADMIN_USER needs to be a string!");
  let admin_password = std::env::var("ADMIN_PASSWORD").expect("ADMIN_PASSWORD needs to be a string!");
  User::new(admin_username, admin_password, true)
}

impl User {
  pub fn new(username: String, clear_text_password: String, admin: bool) -> Self {
    User {
      username,
      password: hash_password(clear_text_password).expect("Was not able to generate a hashed password"),
      admin,
    }
  }
  pub fn from_existing(username: String, hashed_password: String, admin: bool) -> Self {
    User {
      username,
      password: hashed_password,
      admin,
    }
  }
}

#[derive(Clone)]
pub struct UserList {
  list: Vec<User>
}

impl UserList {
  pub fn new() -> Self {
    UserList { list: vec![] }
  }

  pub fn get_all(&self) -> Vec<User> {
    self.list.to_vec()
  }

  pub fn add(&mut self, user_to_add: User) {
    self.list.push(user_to_add)
  }

  pub fn find(&self, name: &str) -> Option<User> {
    self.list.clone().into_iter().find(|user| user.username == name)
  }

  pub fn exists(&self, name: &str) -> bool {
    self.list.iter().any(|u| u.username == name)
  }
}

#[derive(Serialize)]
struct UserListResponse {
  users: Vec<User>
}

#[derive(Serialize)]
struct UserResponse {
  user: User
}

#[derive(Deserialize)]
struct NewUserBody {
  username: String,
  password: String,
}

async fn get_all_users(
  State(state): State<AppState>
) -> (StatusCode, Json<UserListResponse>) {
  let user_list = state.user_list.lock().unwrap();
  let response = UserListResponse {
    users: user_list.get_all()
  };
  (StatusCode::OK, Json(response))
}

async fn find_user(
  State(state): State<AppState>,
  Path(name): Path<String>
) -> Result<(StatusCode, Json<UserResponse>), (StatusCode, String)> {
  let user_list = state.user_list.lock().unwrap();
  let found_user = user_list.find(&name);
  match found_user {
    Some(user) => Ok((StatusCode::OK, Json(UserResponse { user }))),
    None => Err((StatusCode::NOT_FOUND, format!("the user \"{name}\" does not exist!")))
  }
}

async fn add_user(
  State(state): State<AppState>,
  Json(new_user): Json<NewUserBody>,
) -> Result<StatusCode, StatusCode> {
  let mut user_list = state.user_list.lock().unwrap();

  if user_list.exists(&new_user.username) {
    return Err(StatusCode::CONFLICT)
  }

  user_list.add(
    User::new(new_user.username, new_user.password, false)
  );

  Ok(StatusCode::CREATED)
}

pub fn router() -> Router<AppState> {
  Router::new()
    .route("/users", get(get_all_users))
    .route("/users/name/:name", get(find_user))
    .route("/auth/register", post(add_user))
}