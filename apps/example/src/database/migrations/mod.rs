pub mod create_users_table;
pub mod create_roles_table;
pub mod create_user_profiles_table;
pub mod create_permissions_table;
pub mod create_files_table;

use lara_db::migrations::MigrationRunner;
use create_users_table::CreateUsersTable;
use create_roles_table::CreateRolesTable;
use create_user_profiles_table::CreateUserProfilesTable;
use create_permissions_table::CreatePermissionsTable;
use create_files_table::CreateFilesTable;

pub fn all_migrations() -> MigrationRunner {
    MigrationRunner::new()
        .add(CreateUsersTable)
        .add(CreateRolesTable)
        .add(CreateUserProfilesTable)
        .add(CreatePermissionsTable)
        .add(CreateFilesTable)
}
