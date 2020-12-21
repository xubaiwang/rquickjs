use super::resolve_simple;
use crate::{Ctx, Error, Loaded, Loader, Module, Resolver, Result, Script};
use std::ops::Deref;

pub trait HasByteCode<'bc> {
    fn get_bytecode(&self) -> &'bc [u8];
}

impl<'bc> HasByteCode<'bc> for &'bc [u8] {
    fn get_bytecode(&self) -> &'bc [u8] {
        self
    }
}

pub type ScaBundleData<D> = &'static [(&'static str, D)];

#[cfg(feature = "phf")]
pub type PhfBundleData<D> = &'static phf::Map<&'static str, D>;

/// The resolver and loader for compiled bundle
#[derive(Debug, Clone, Copy)]
pub struct Bundle<T>(pub T);

impl<T> Deref for Bundle<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<D> Resolver for Bundle<ScaBundleData<D>> {
    fn resolve<'js>(&mut self, _ctx: Ctx<'js>, base: &str, name: &str) -> Result<String> {
        let path = resolve_simple(base, name);
        if self.iter().any(|(name, _)| *name == path) {
            Ok(path)
        } else {
            Err(Error::new_resolving(base, name))
        }
    }
}

#[cfg(feature = "phf")]
impl Resolver for Bundle<PhfBundleData<D>> {
    fn resolve<'js>(&mut self, _ctx: Ctx<'js>, base: &str, name: &str) -> Result<String> {
        let path = resolve_simple(base, name);
        if self.contains_key(path.as_str()) {
            Ok(path)
        } else {
            Err(Error::new_resolving(base, name))
        }
    }
}

impl<D> Loader<Script> for Bundle<ScaBundleData<D>>
where
    D: HasByteCode<'static>,
{
    fn load<'js>(&mut self, ctx: Ctx<'js>, name: &str) -> Result<Module<'js, Loaded<Script>>> {
        self.iter()
            .find(|(module_name, _)| *module_name == name)
            .ok_or_else(|| Error::new_loading(name))
            .and_then(|(_, bytecode)| Module::read_object_const(ctx, bytecode.get_bytecode()))
    }
}

#[cfg(feature = "phf")]
impl Loader<Script> for Bundle<PhfBundleData> {
    fn load<'js>(&mut self, ctx: Ctx<'js>, name: &str) -> Result<Module<'js, Loaded<Script>>> {
        self.get(name)
            .ok_or_else(|| Error::new_loading(name))
            .and_then(|buf| Module::read_object_const(ctx, buf))
    }
}
