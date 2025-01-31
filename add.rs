pub struct Add(char);

impl undo::Action for Add {
    type Target = String;
    type Output = ();
    type Error = &'static str;

    fn apply(&mut self, s: &mut String) -> undo::Result<Add> {
        s.push(self.0);
        Ok(())
    }

    fn undo(&mut self, s: &mut String) -> undo::Result<Add> {
        self.0 = s.pop().ok_or("s is empty")?;
        Ok(())
    }
}
