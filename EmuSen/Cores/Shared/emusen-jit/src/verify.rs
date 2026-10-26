//! The verifier's shape: a reference machine, the processor's interpreter, stepped once for every instruction the blocks run and
//! compared after each. The processor says what a step is and what is compared. See EmuSen_Jit.md §4.

/// The interpreter's machine, as a processor defines it.
pub trait Reference {
    /// The machine the blocks run, as the comparison borrows it.
    type Subject<'a>: Copy;
    /// One interpreter step.
    fn step(&mut self);
    /// Everything a step can change, compared; the first difference described, or none.
    fn differs(&self, subject: Self::Subject<'_>) -> Option<String>;
    /// The rest of the machine, compared at the end of a run; the difference described, or none.
    fn differs_whole(&self, subject: Self::Subject<'_>) -> Option<String>;
}

/// The reference machine and the steps it has taken, a step behind or level with the machine the blocks run.
pub struct Verifier<R> {
    pub reference: R,
    pub steps: u64,
}

impl<R: Reference> Verifier<R> {
    pub fn new(reference: R) -> Verifier<R> {
        Verifier { reference, steps: 0 }
    }

    /// One interpreter step, then the two compared; `what` names the path the blocks took.
    pub fn follow(&mut self, subject: R::Subject<'_>, what: &str) {
        self.reference.step();
        self.steps += 1;
        self.check(subject, what);
    }

    /// The two compared without a step, for a path both machines ran by the same code.
    pub fn check(&self, subject: R::Subject<'_>, what: &str) {
        if let Some(difference) = self.reference.differs(subject) {
            panic!("the recompiler parted from the interpreter after {} steps, after {what}: {difference}", self.steps);
        }
    }

    /// The whole machine, made at the end of a run.
    pub fn check_all(&self, subject: R::Subject<'_>) {
        self.check(subject, "the end of the run");
        if let Some(difference) = self.reference.differs_whole(subject) {
            panic!("the recompiler's machine parted from the interpreter's after {} steps, {difference}", self.steps);
        }
    }
}
