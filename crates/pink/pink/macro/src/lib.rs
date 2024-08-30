use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use syn::{parse_macro_input, Result};

mod chain_extension;
mod contract;
mod driver_system;

/// A drop-in replacement for `ink::contract` with pink-specific feature extensions.
///
/// # pink-specific features
/// - `#[pink(on_block_end)]`
///   Marks a function as being called on each phala block has been dispatched.
#[proc_macro_attribute]
pub fn contract(arg: TokenStream, input: TokenStream) -> TokenStream {
    let config = parse_macro_input!(arg as TokenStream2);
    let module = parse_macro_input!(input as TokenStream2);
    let module = contract::patch(module, config);
    module.into()
}

/// Internal use only.
#[proc_macro_attribute]
pub fn chain_extension(_: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as TokenStream2);
    let output = chain_extension::patch(input);
    output.into()
}

/// Mark an ink trait as pink's system contract. Internal use only.
#[proc_macro_attribute]
pub fn system(arg: TokenStream, input: TokenStream) -> TokenStream {
    let config = parse_macro_input!(arg as TokenStream2);
    let module = parse_macro_input!(input as TokenStream2);
    let module = driver_system::patch(module, config, driver_system::InterfaceType::System);
    module.into()
}

/// This procedural macro marks an ink! trait as a 'driver contract' for the Pink system.
///
/// # What is the driver system?
///
/// The driver system is a straightforward concept. In the Pink system, there is a registry mapping driver names
/// to driver contract addresses. The System contract provides two methods to manage this registry:
/// `System::set_driver(driver_name, contract_address)` and `System::get_driver(driver_name)`.
///
/// # How does this macro work?
///
/// When this attribute is used, it modifies the given trait to be utilized as a driver contract
/// within the Pink system. This is achieved by adding a new type, TraitNameRef, which implements
/// the marked trait and provides a static method `instance()` to retrieve an instance of the driver.
///
/// # Example
///
/// Below, the `SidevmOperation` trait is annotated with `#[pink::driver]`. This marks it
/// as a driver contract, enabling it to manage SideVM deployments.
///
/// ```ignore
/// #[pink::driver]
/// #[ink::trait_definition]
/// pub trait SidevmOperation {
///     #[ink(message)]
///     fn deploy(&self, code_hash: Hash) -> Result<(), DriverError>;
///
///     #[ink(message)]
///     fn can_deploy(&self, contract_id: AccountId) -> bool;
/// }
/// ```
///
/// Once a trait has been defined as a driver using this macro, it can be set as a driver
/// by invoking the `System::set_driver(driver_name, contract_address)` method.
///
/// # Usage
///
/// The actual driver can then be retrieved and its methods, defined by the trait, can be used.
/// For instance, to start a SideVM, one would get the driver instance and call its `deploy` method:
///
/// ```ignore
/// pub fn start_sidevm(code_hash: Hash) -> Result<(), system::DriverError> {
///     let driver =
///         SidevmOperationRef::instance().ok_or(system::Error::DriverNotFound)?;
///     driver.deploy(code_hash)
/// }
/// ```
///
/// Here, `SidevmOperationRef::instance()` retrieves an instance of the driver contract for "SidevmOperation",
/// and then the `deploy` method of the driver is invoked to deploy a SideVM instance.
/// Internally, `SidevmOperationRef::instance()` retrieves the driver contract by invoking `System::get_driver("SidevmOperation")`.
///
/// Note: The name of the driver contract instance (e.g., "SidevmOperation") is generated by the macro.
#[proc_macro_attribute]
pub fn driver(arg: TokenStream, input: TokenStream) -> TokenStream {
    let config = parse_macro_input!(arg as TokenStream2);
    let module = parse_macro_input!(input as TokenStream2);
    let module = driver_system::patch(module, config, driver_system::InterfaceType::Driver);
    module.into()
}

#[cfg(not(test))]
fn find_crate_name(origin: &str) -> Result<syn::Ident> {
    use proc_macro2::Span;
    use proc_macro_crate::{crate_name, FoundCrate};
    let name = match crate_name(origin) {
        Ok(FoundCrate::Itself) => syn::Ident::new("crate", Span::call_site()),
        Ok(FoundCrate::Name(alias)) => syn::Ident::new(&alias, Span::call_site()),
        Err(e) => {
            return Err(syn::Error::new(Span::call_site(), &e));
        }
    };
    Ok(name)
}

#[cfg(test)]
fn find_crate_name(origin: &str) -> Result<syn::Ident> {
    use heck::ToSnakeCase;
    use proc_macro2::Span;
    Ok(syn::Ident::new(&origin.to_snake_case(), Span::call_site()))
}