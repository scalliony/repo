use std::collections::BTreeMap;
use bulb::{hex::Hex, dto::{UserId, Str, ProgramId}};
use super::bot::Program;

#[derive(Default)]
pub struct Users(Vec<User>, BTreeMap<Str, UserId>);
impl Users {
    pub fn new() -> Self {
        Self(Vec::new(), BTreeMap::new())
    }
    #[inline]
    pub fn by_login(&self, login: &Str) -> Option<(UserId, &User)> {
        self.1.get(login).map(|id| (*id, &self.0[usize::from(*id)]))
    }
    pub fn set(&mut self, data: User) -> UserId {
        if let Some(id) = self.1.get(&data.login) {
            self.0[usize::from(*id)] = data;
            *id
        } else {
            let id = UserId::from(self.0.len());
            self.1.insert(data.login.clone(), id);
            self.0.push(data);
            id
        }
    }
    #[inline]
    pub fn get(&self, id: UserId) -> Option<&User> {
        self.0.get(usize::from(id))
    }
    #[inline]
    pub fn get_mut(&mut self, id: UserId) -> Option<&mut User> {
        self.0.get_mut(usize::from(id))
    }
}

pub struct User {
    login: Str,
    name: Str,
    pub spawn: Hex,
    programs: Vec<Program>,
}
impl User {
    pub fn new(login: Str, name: Str, spawn: Hex) -> Self {
        Self { login, name, spawn, programs: Vec::new() }
    }

    #[inline]
    pub fn login(&self) -> &Str {
        &self.login
    }
    #[inline]
    pub fn name(&self) -> &Str {
        &self.name
    }
    #[inline]
    pub fn get_program(&self, id: ProgramId) -> Option<&Program> {
        self.programs.get(id as usize)
    }
    pub fn add_program(&mut self, prg: Program) -> ProgramId {
        let id = self.programs.len();
        self.programs.push(prg);
        id as ProgramId
    }
}
