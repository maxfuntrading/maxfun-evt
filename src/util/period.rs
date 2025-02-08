use sea_orm::EnumIter;


#[derive(Copy, Clone, Debug, EnumIter)]
pub enum PeriodType {
    // 10秒钟
    S10,
    // 5分钟
    M5,
    // 30分钟
    M30,
    // 2小时
    H2,
    // 1天
    D1,
    // // 1周
}

impl PeriodType {
    pub fn open_ts(&self, ts: i64) -> i64 {
        ts - ts % self.period()
    }
    // pub fn last_open_ts(&self, ts: i64) -> i64 {
    //     self.open_ts(ts) - self.period()
    // }

    pub fn period(&self) -> i64 {
        match self {
            PeriodType::S10 => 10,
            PeriodType::M5 => 300,
            PeriodType::M30 => 1800,
            PeriodType::H2 => 7200,
            PeriodType::D1 => 86400,
        }
    }

    pub fn close_ts(&self, open_ts: i64) -> i64 {
        open_ts + self.period() - 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn t_alias_open_ts() {
        assert_eq!(PeriodType::S10.open_ts(1702696269), 1702696200);
        // assert_eq!(alias_open_ts(1702696269, super::PeriodType::Min15), 1702695600);
        // assert_eq!(alias_open_ts(1702696269, super::PeriodType::H1), 1702695600);
        // assert_eq!(alias_open_ts(1702696269, super::PeriodType::D1), 1702684800);
        // assert_eq!(alias_open_ts(1702696269, super::PeriodType::W1), 1702252800);
    }
}