use crate::api::routes::AppState;
use axum::{
    extract::State,
    Json, Router,
    routing::{get, post},
};
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};
use rusqlite::{params, Row};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

/// JWT 配置
const JWT_SECRET: &[u8] = b"quantrust_secret_key_change_in_production";
const JWT_EXPIRATION_HOURS: i64 = 24 * 7; // 7 days

/// 创建 auth 路由
pub fn create_auth_router(state: AppState) -> Router {
    Router::new()
        .route("/api/auth/register", post(register))
        .route("/api/auth/login", post(login))
        .route("/api/auth/me", get(me))
        .with_state(state)
}

// ============ 数据模型 ============

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
    pub email: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    pub user: UserInfo,
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserInfo {
    pub id: i64,
    pub username: String,
    pub email: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // user_id
    pub username: String,
    pub exp: i64,
    pub iat: i64,
}

// ============ JWT 函数 ============

fn create_jwt(user_id: i64, username: &str) -> Result<String, String> {
    let now = Utc::now();
    let exp = (now + Duration::hours(JWT_EXPIRATION_HOURS)).timestamp();

    let claims = Claims {
        sub: user_id.to_string(),
        username: username.to_string(),
        exp,
        iat: now.timestamp(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(JWT_SECRET),
    )
        .map_err(|e| format!("Failed to create token: {}", e))
}

fn verify_jwt(token: &str) -> Result<TokenData<Claims>, String> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(JWT_SECRET),
        &Validation::default(),
    )
        .map_err(|e| format!("Invalid token: {}", e))
}

fn row_to_user(row: &Row) -> rusqlite::Result<UserInfo> {
    Ok(UserInfo {
        id: row.get(0)?,
        username: row.get(1)?,
        email: row.get(2)?,
        created_at: row.get(3)?,
    })
}

// ============ 路由处理器 ============

/// 用户注册
async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<serde_json::Value>, String> {
    // 验证输入
    if req.username.len() < 3 || req.username.len() > 32 {
        return Err("用户名长度必须在 3-32 个字符之间".to_string());
    }
    if req.password.len() < 6 {
        return Err("密码长度至少 6 个字符".to_string());
    }

    // 哈希密码
    let hashed_password = hash(&req.password, DEFAULT_COST)
        .map_err(|e| format!("密码加密失败: {}", e))?;

    // 插入数据库
    let now = Utc::now().to_rfc3339();
    
    let db = state.db.lock().map_err(|e| format!("数据库锁失败: {}", e))?;
    
    let result = db.execute(
        "INSERT INTO users (username, password, email, created_at) VALUES (?1, ?2, ?3, ?4)",
        params![req.username, hashed_password, req.email, now],
    );

    match result {
        Ok(_) => {
            // 获取刚插入的用户 ID
            let user_id = db.last_insert_rowid();
            
            // 生成 JWT
            let token = create_jwt(user_id, &req.username)?;
            
            let user = UserInfo {
                id: user_id,
                username: req.username,
                email: req.email,
                created_at: now,
            };

            Ok(Json(serde_json::json!({
                "success": true,
                "data": AuthResponse { user, token },
                "message": "注册成功"
            })))
        }
        Err(e) => {
            if e.to_string().contains("UNIQUE constraint failed") {
                Err("用户名已存在".to_string())
            } else {
                Err(format!("注册失败: {}", e))
            }
        }
    }
}

/// 用户登录
async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<serde_json::Value>, String> {
    // 查询用户
    let db = state.db.lock().map_err(|e| format!("数据库锁失败: {}", e))?;
    
    let user_result: Result<UserInfo, _> = db.query_row(
        "SELECT id, username, email, created_at FROM users WHERE username = ?1",
        params![req.username],
        row_to_user,
    );

    let user = match user_result {
        Ok(u) => u,
        Err(_) => return Err("用户名或密码错误".to_string()),
    };

    // 验证密码
    let password_hash: String = db.query_row(
        "SELECT password FROM users WHERE id = ?1",
        params![user.id],
        |row| row.get(0),
    ).map_err(|e| format!("数据库查询失败: {}", e))?;

    let valid = verify(&req.password, &password_hash)
        .map_err(|e| format!("密码验证失败: {}", e))?;

    if !valid {
        return Err("用户名或密码错误".to_string());
    }

    // 生成 JWT
    let token = create_jwt(user.id, &user.username)?;

    Ok(Json(serde_json::json!({
        "success": true,
        "data": AuthResponse { user, token },
        "message": "登录成功"
    })))
}

/// 获取当前用户信息
async fn me(
    State(state): State<AppState>,
    axum::extract::Json(claims): axum::extract::Json<Claims>,
) -> Result<Json<serde_json::Value>, String> {
    let user_id: i64 = claims.sub.parse().map_err(|_| "无效的用户ID")?;

    let db = state.db.lock().map_err(|e| format!("数据库锁失败: {}", e))?;
    
    let user = db.query_row(
        "SELECT id, username, email, created_at FROM users WHERE id = ?1",
        params![user_id],
        row_to_user,
    ).map_err(|_| "用户不存在")?;

    Ok(Json(serde_json::json!({
        "success": true,
        "data": user,
        "message": "ok"
    })))
}
