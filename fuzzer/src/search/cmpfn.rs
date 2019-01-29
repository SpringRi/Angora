use super::*;
use angora_common::tag::TagSeg;
pub struct FnFuzz<'a> {
    handler: SearchHandler<'a>,
}

impl<'a> FnFuzz<'a> {
    pub fn new(handler: SearchHandler<'a>) -> Self {
        Self { handler }
    }

    fn insert_bytes(&mut self, n: usize) {
        debug!("add {} bytes", n);
        let last = self.handler.cond.offsets.last().unwrap();
        let off = last.begin as usize;
        let mut end = last.end;
        let v = self.handler.buf[off];
        for _ in 0..n {
            self.handler.buf.insert(off, v);
            let begin = end;
            end = begin + 1;
            self.handler.cond.offsets.push(TagSeg {
                sign: false,
                begin,
                end,
            })
        }
    }

    fn remove_bytes(&mut self, n: usize) {
        debug!("remove {} bytes", n);
        for _ in 0..n {
            let last = self.handler.cond.offsets.last().unwrap();
            let off = last.begin as usize;
            let size = last.end as usize - off;
            self.handler.buf.remove(off);
            if size > 1 {
                self.handler.cond.offsets.last_mut().unwrap().end = last.end - 1;
            } else {
                self.handler.cond.offsets.pop();
            }
        }
    }

    pub fn run(&mut self) {
        let len = self.handler.cond.base.size as usize;
        let output = self.handler.cond.variables.split_off(len);
        let off_len = self.handler.cond.offsets.len();
        if off_len != output.len() {
            error!("not all bytes are tainted");
        }
        if off_len < len {
            self.insert_bytes(len - off_len);
        }
        if off_len > len {
            self.remove_bytes(off_len - len);
        }

        let mut input = self.handler.get_f_input();
        let input_vals = input.get_value();
        if input_vals.len() == len {
            for i in 0..len {
                let diff = output[i] as i16 - input_vals[i] as i16;
                if diff != 0 {
                    debug!("has diff");
                }
                self.handler.cond.variables[i] =
                    (self.handler.cond.variables[i] as i16 - diff) as u8;
            }
        }
        input.assign(&self.handler.cond.variables);
        self.handler.execute_input(&input);

        self.handler.cond.mark_as_done();
    }
}
