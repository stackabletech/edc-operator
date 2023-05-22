# IONOS S3 Core
---

 Contains the API of the extension with all the allowed operations (connection, creation of buckets, upload of files, etc...) related to the S3 storage.

 It is required to configure an `Access key` and a `Secret Access Key` from the IONOS S3 storage service.

 ## Configuration

The credentials can be found/configured in one of the following:
- Vault;
- Properties file;
- Java arguments;
- Environment Variables (`IONOS_ACCESS_KEY`, `IONOS_SECRET_KEY` and `IONOS_TOKEN`);

It is required to configure those parameters:

| Parameter name                          | Description                            | Mandatory  |
|-----------------------------------------|----------------------------------------| ---------- |
| `edc.ionos.access.key`                    | IONOS Access Key Id to access S3     | Yes if the context is accessing file |
| `edc.ionos.secret.access.key`             | IONOS Secret Access Key to access S3 | Yes if the context is accessing file |
| `edc.ionos.token`                         | IONOS token to allow S3 provisioning | Yes if the context is provisioning access for others |

To create the token please take a look at the following [documentation](../../ionos_token.md).
