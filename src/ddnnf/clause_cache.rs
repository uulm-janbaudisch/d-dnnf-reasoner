use std::collections::BTreeSet;

use tempfile::Builder;

use crate::{
    parser::{build_ddnnf, persisting::write_cnf_to_file},
    Ddnnf,
};

#[derive(Debug, Clone, Default)]
/// Represents all types of Nodes with its different parts
pub struct ClauseCache {
    /// All clauses corresponding to the CNF. Is empty if ddnnife did not get clauses as input.
    clauses: BTreeSet<BTreeSet<i32>>,
    /// The clauses that have to be added to result in the parent d-DNNF.
    edit_add: Vec<BTreeSet<i32>>,
    /// The clauses that have to be removed to result in the parent d-DNNF.
    edit_rmv: Vec<BTreeSet<i32>>,
    /// The number of features after applying the edit.
    total_features: Option<u32>,
    /// The number of features before the edit.
    old_total_features: Option<u32>,
    /// An old cached d-DNNF state that can be swaped with the currently used one.
    old_state: Box<Ddnnf>,
}

impl ClauseCache {
    /// Updates the ClauseCache with starting values.
    pub fn initialize(
        &mut self,
        ddnnf: Ddnnf,
        clauses: BTreeSet<BTreeSet<i32>>,
        total_features: Option<u32>,
    ) {
        self.old_state = Box::new(ddnnf);
        self.clauses = clauses;
        self.old_total_features = total_features;
    }

    /// Applies edit operations on the set of clauses by adding and removing clauses.
    /// Returns if the manipulation was succesful (we cannot remove non existing clauses)
    fn setup_for_edit(
        &mut self,
        add: Vec<BTreeSet<i32>>,
        rmv: Vec<BTreeSet<i32>>,
        total: Option<u32>,
    ) -> bool {
        let old_set_clauses = self.clauses.clone();
        for clause in self.edit_rmv.iter() {
            if !self.clauses.remove(clause) {
                // revert made changes if any clause could not be found
                self.clauses = old_set_clauses;
                return false;
            }
        }
        for clause in self.edit_add.clone().into_iter() {
            self.clauses.insert(clause);
        }

        self.edit_add = add;
        self.edit_rmv = rmv;
        self.total_features = total;
        true
    }

    /// Sets up the edit operations for an undo operation by applying and flipping added and removed clauses.
    pub fn setup_for_undo(&mut self) -> bool {
        self.setup_for_edit(
            self.edit_rmv.clone(),
            self.edit_add.clone(),
            self.old_total_features,
        )
    }

    pub fn apply_edits_and_replace(
        &mut self,
        add: Vec<BTreeSet<i32>>,
        rmv: Vec<BTreeSet<i32>>,
        total: u32,
    ) -> bool {
        if self.total_features.is_none() || !self.setup_for_edit(add, rmv, Some(total)) {
            return false;
        }

        // Create a temporary file named "temp.cnf"
        let temp_file = Builder::new()
            .prefix("temp")
            .suffix(".cnf")
            .tempfile()
            .expect("Failed to create a temporary file");

        write_cnf_to_file(
            &self.clauses,
            self.total_features.unwrap(),
            temp_file
                .path()
                .to_str()
                .expect("Failed to convert path to string while trying to save as CNF!"),
        )
        .expect("Failed to save updated CNF to file");

        self.old_state = Box::new(build_ddnnf("temp.cnf", self.total_features));
        true
    }

    pub fn get_old_state(self) -> Box<Ddnnf> {
        self.old_state
    }
}
