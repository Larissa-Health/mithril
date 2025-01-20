resource "google_storage_bucket" "cloud_storage" {
  name          = local.mithril_aggregator_cdn_host
  location      = var.google_region
  force_destroy = var.google_storage_bucket_force_destroy

  depends_on = [googlesiteverification_dns.mithril-aggregator]

  lifecycle_rule {
    condition {
      age            = var.google_storage_bucket_max_age
      matches_prefix = var.google_storage_bucket_prefix_with_lifecyle_rule
    }
    action {
      type = "Delete"
    }
  }
}

resource "google_service_account" "cloud_storage" {
  account_id   = local.environment_name
  display_name = "${local.environment_name}-cs-sa"
  description  = "${local.environment_name} cloud storage service account"
}

resource "google_service_account_key" "cloud_storage" {
  service_account_id = google_service_account.cloud_storage.name
  public_key_type    = "TYPE_X509_PEM_FILE"
}

locals {
  google_cloud_storage_credentials_json = base64decode(google_service_account_key.cloud_storage.private_key)
}

resource "google_storage_bucket_iam_member" "cloud_storage_viewer" {
  bucket = google_storage_bucket.cloud_storage.name
  role   = "roles/storage.objectViewer"
  member = "serviceAccount:${google_service_account.cloud_storage.email}"
}

resource "google_storage_bucket_iam_member" "cloud_storage_creator" {
  bucket = google_storage_bucket.cloud_storage.name
  role   = "roles/storage.objectCreator"
  member = "serviceAccount:${google_service_account.cloud_storage.email}"
}

resource "google_storage_bucket_iam_member" "legacy_bucket_writer" {
  bucket = google_storage_bucket.cloud_storage.name
  role   = "roles/storage.legacyBucketWriter"
  member = "serviceAccount:${google_service_account.cloud_storage.email}"
}
