use std::fs::{self, File};
use std::io::{Read, Write};
use std::sync::Arc;
use rspotify::{
    clients::OAuthClient, scopes, AuthCodeSpotify, Config, Credentials, OAuth, Token, TokenCallback,
};

pub async fn auth() {
    let creds = Credentials::from_env().unwrap();
    let oauth = OAuth::from_env(scopes!(
        "user-read-playback-state",
        "user-modify-playback-state",
        "user-follow-modify",
        "playlist-modify-public",
        "playlist-modify-private",
        "user-library-modify"
    ))
    .unwrap();

    let write_token_to_file = |token: Token| {
        let config_path = dirs::home_dir().expect("Unable to find home directory").join(".config/spotify-cli");
        fs::create_dir_all(&config_path).expect("Unable to create config directory");
        let token_path = config_path.join(".token");
        let mut file = File::create(&token_path).unwrap();

        let serialized = serde_json::to_string(&token).unwrap();
        let _ = file.write_all(serialized.to_string().as_bytes());
        println!(">>> Succesfully wrote token to file in {}!", token_path.display());
        Ok(())
    };
    let token_callback = TokenCallback(Box::new(write_token_to_file));

    // Enabling automatic token refreshing in the config
    let config = Config {
        token_callback_fn: Arc::new(Some(token_callback)),
        ..Default::default()
    };

    println!(">>> Fetch token with AuthCodeSpotify");
    let spotify = AuthCodeSpotify::with_config(creds, oauth, config);
    let url = spotify.get_authorize_url(false).unwrap();

    // This function requires the `cli` feature enabled.
    spotify
        .prompt_for_token(&url)
        .await
        .expect("couldn't authenticate successfully");

    println!(">>> authentication completed!");
}

pub fn spotify_from_token() -> AuthCodeSpotify {
    let config_path = dirs::home_dir().expect("Unable to find home directory").join(".config/spotify-cli");
    let token_path = config_path.join(".token");
    let mut file = File::open(&token_path).expect(
        &format!("couldn't find .token file in {}, maybe try running 'spotify auth' first?", config_path.display())
    );
    let mut contents = String::new();
    let _ = file.read_to_string(&mut contents);
    let token = serde_json::from_str(&contents).unwrap();

    AuthCodeSpotify::from_token(token)
}
