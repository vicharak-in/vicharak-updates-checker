pub mod update;
use update::Package;
use update::VaamanPackages;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let available_packages = vec!["vaaman-aes", "vaamos-menu-git"];
    let mut packages = VaamanPackages::new();

    for package in available_packages {
        packages.add_package(Package::new(
            package.to_string(),
            Package::get_latest_version(package).await?,
        ));
    }

    let update = packages.check_updates()?;

    if update.is_empty() {
        println!("No updates available");
    } else {
        println!("Updates available: {update:?}");

        for package in update {
            println!("Updating {}", package.name);
            package.update_package()?;
        }
    }

    packages.save_packages_to_file()?;

    Ok(())
}
