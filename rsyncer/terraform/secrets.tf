# secrets.tf
# This file manages secret environment variables for Cloud Run using Google Secret Manager


resource "google_secret_manager_secret" "rspotify_client_id" {
  secret_id = "RSPOTIFY_CLIENT_ID"
  replication {
    auto {}
  }
}

resource "google_secret_manager_secret_version" "rspotify_client_id_version" {
  secret      = google_secret_manager_secret.rspotify_client_id.id
  secret_data = var.RSPOTIFY_CLIENT_ID
}

resource "google_secret_manager_secret" "rspotify_client_secret" {
  secret_id = "RSPOTIFY_CLIENT_SECRET"
  replication {
    auto {}
  }
}

resource "google_secret_manager_secret_version" "rspotify_client_secret_version" {
  secret      = google_secret_manager_secret.rspotify_client_secret.id
  secret_data = var.RSPOTIFY_CLIENT_SECRET
}

resource "google_secret_manager_secret" "lastfm_api_key" {
  secret_id = "LASTFM_API_KEY"
  replication {
    auto {}
  }
}

resource "google_secret_manager_secret_version" "lastfm_api_key_version" {
  secret      = google_secret_manager_secret.lastfm_api_key.id
  secret_data = var.LASTFM_API_KEY
}

resource "google_secret_manager_secret" "lastfm_api_secret" {
  secret_id = "LASTFM_API_SECRET"
  replication {
    auto {}
  }
}

resource "google_secret_manager_secret_version" "lastfm_api_secret_version" {
  secret      = google_secret_manager_secret.lastfm_api_secret.id
  secret_data = var.LASTFM_API_SECRET
}

# Grant Cloud Run SA read access to secrets
resource "google_secret_manager_secret_iam_member" "cloudrun_sa_secrets_reader" {
  for_each = {
    rspotify_client_id     = google_secret_manager_secret.rspotify_client_id.id
    rspotify_client_secret = google_secret_manager_secret.rspotify_client_secret.id
    lastfm_api_key         = google_secret_manager_secret.lastfm_api_key.id
    lastfm_api_secret      = google_secret_manager_secret.lastfm_api_secret.id
  }
  secret_id = each.value
  role      = "roles/secretmanager.secretAccessor"
  member    = "serviceAccount:${google_service_account.cloud_run.email}"
}
