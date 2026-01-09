use crate::db;
use crate::db::session::entity::SessionEntity;
use crate::db::user::entity::UserEntity;
use crate::error::auth::AuthError;
use crate::utils::jwt;
use crate::utils::telegram::verify_telegram_login;
use fluent_templates::LanguageIdentifier;
use google_cloud_auth::credentials::idtoken::verifier;
use sqlx::PgPool;
use time::OffsetDateTime;
use crate::error::AppError;

pub async fn logout(
    db: &PgPool,
    user_id: i64
) -> Result<(), AppError> {
    db::session::repo::delete(
        db,
        user_id
    ).await?;
    Ok(())
}

pub async fn get_me(
    db: &PgPool,
    user_id: i64,
    lang: LanguageIdentifier
) -> Result<UserEntity, AppError> {
    Ok(
        db::user::repo::find_by_id(db, user_id)
            .await?.ok_or(AppError::NotFound(lang))?
    )
}

pub async fn refresh_tokens(
    db: &PgPool,
    refresh_token: String,
    user_agent: String,
    role: String,
    lang: LanguageIdentifier
) -> Result<(String, String), AppError> {
    let claims = jwt::validate_refresh_token(&refresh_token)
        .map_err(|_| AuthError::Unauthorized(lang.clone()))?;
    if claims.role != role { return Err(AuthError::Unauthorized(lang).into()); }

    let session = db::session::repo::find_by_refresh_token_hash(
        db,
        jwt::hash_token(&refresh_token)
    ).await?.ok_or(AuthError::Unauthorized(lang.clone()))?;
    if let Some(agent) = session.user_agent && agent != user_agent {
        return Err(AuthError::Unauthorized(lang).into());
    }

    let (refresh_token, access_token) = jwt::generate(session.user_id, &role)
        .map_err(|_| AuthError::Unauthorized(lang.clone()))?;

    db::session::repo::update(
        db,
        SessionEntity{
            id: session.id,
            user_id: session.user_id,
            refresh_token_hash: jwt::hash_token(&refresh_token),
            user_agent: Some(user_agent),
            created_at: session.created_at,
            last_used_at: OffsetDateTime::now_utc()
        }
    ).await?;

    Ok((refresh_token, access_token))
}


pub async fn signin_with_telegram(
    db: &PgPool,
    user_agent: String,
    role: String,
    data: String,
    lang: LanguageIdentifier
) -> Result<(UserEntity, String, String), AppError>{
    let auth_data = verify_telegram_login(
        &data,
        &crate::utils::CONFIG.telegram_bot_token,
    )?;

    let entity = db::user::repo::find_by_telegram_id(db, auth_data.id)
        .await?;

    let mut tx = db.begin().await?;
    let user = UserEntity {
        id: 1,
        first_name: auth_data.first_name,
        last_name: auth_data.last_name,
        avatar_path: auth_data.photo_url,
        email: None,
        telegram_id: Some(auth_data.id),
        telegram_username: auth_data.username,
        google_id: None,
        password_hash: None,
        is_admin: false,
        created_at: OffsetDateTime::now_utc(),
        updated_at: OffsetDateTime::now_utc(),
    };
    let entity = if let Some(_) = entity {
        db::user::repo::update(&mut tx, user).await?
    } else {
        db::user::repo::insert(&mut tx, user).await?
    };

    let (refresh_token, access_token) = jwt::generate(entity.id, &role)
        .map_err(|_| AuthError::Unauthorized(lang.clone()))?;
    db::session::repo::insert(
        &mut tx,
        SessionEntity{
            id: 1,
            user_id: entity.id,
            refresh_token_hash: jwt::hash_token(&refresh_token),
            user_agent: Some(user_agent),
            created_at: OffsetDateTime::now_utc(),
            last_used_at: OffsetDateTime::now_utc()
        }
    ).await?;
    tx.commit().await?;

    Ok((entity, refresh_token, access_token))
}

pub async fn signin_with_google(
    db: &PgPool,
    user_agent: String,
    role: String,
    data: String,
    lang: LanguageIdentifier
) -> Result<(UserEntity, String, String), AppError>{
    let verifier = verifier::Builder::new(vec!["22454749576-42ii04497d5aceqndkbvpnvn29nvub02.apps.googleusercontent.com"])
            .build();
    let auth_data = verifier.verify(&data)
        .await
        .map_err(|_| AuthError::Unauthorized(lang.clone()))?;

    let id = auth_data["sub"].as_str().unwrap();
    let name = auth_data["name"].as_str().unwrap();
    let family_name = auth_data["family_name"].as_str();
    let picture = auth_data["picture"].as_str();
    let email = auth_data["email"].as_str();

    let entity = db::user::repo::find_by_google_id(
        db,
        id
    ).await?;

    let mut tx = db.begin().await?;
    let user = UserEntity {
        id: 1,
        first_name: name.to_string(),
        last_name: family_name.map(|s| s.to_string()),
        avatar_path: picture.map(|s| s.to_string()),
        email: email.map(|s| s.to_string()),
        telegram_id: None,
        telegram_username: None,
        google_id: Some(id.to_string()),
        password_hash: None,
        is_admin: false,
        created_at: OffsetDateTime::now_utc(),
        updated_at: OffsetDateTime::now_utc(),
    };
    let entity = if entity.is_some() {
        db::user::repo::update(&mut tx, user).await?
    } else {
        db::user::repo::insert(&mut tx, user).await?
    };

    let (refresh_token, access_token) = jwt::generate(entity.id, &role)
        .map_err(|_| AuthError::Unauthorized(lang.clone()))?;
    db::session::repo::insert(
        &mut tx,
        SessionEntity{
            id: 1,
            user_id: entity.id,
            refresh_token_hash: jwt::hash_token(&refresh_token),
            user_agent: Some(user_agent),
            created_at: OffsetDateTime::now_utc(),
            last_used_at: OffsetDateTime::now_utc()
        }
    ).await?;
    tx.commit().await?;

    Ok((entity, refresh_token, access_token))
}
