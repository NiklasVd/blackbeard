pub const BASE_SHIP_ESCUDO_BALANCE: u32 = 100;
pub const BASE_ESCUDO_PAYOUT: u32 = 30;
pub const NETWORTH_PAYOUT_PERCENTAGE: f32 = 0.1;

pub struct Deposit {
    pub balance: u32,
    pub networth: u32
}

impl Deposit {
    pub fn new(balance: u32) -> Deposit {
        Deposit {
            balance, networth: balance
        }
    }
    pub fn default() -> Deposit {
        Self::new(BASE_SHIP_ESCUDO_BALANCE)
    }

    pub fn add(&mut self, val: u32) {
        self.balance += val;
        self.networth += val;
    }

    pub fn spend(&mut self, val: u32) {
        self.balance -= val;
        // Removal due to expenditure does not decrease net worth
    }

    pub fn lose(&mut self, val: u32) {
        self.spend(val);
        self.networth -= val;
    }
}

pub struct Economy {
    pub escudos_in_circulation: u32,
    pub produced_escudos: u32,
    pub deposits: u32
}

impl Economy {
    pub fn new() -> Economy {
        Economy {
            escudos_in_circulation: 0, produced_escudos: 0, deposits: 0
        }
    }

    pub fn add_deposit(&mut self) {
        self.deposits += 1;
        self.escudos_in_circulation += BASE_SHIP_ESCUDO_BALANCE;
    }

    pub fn produce(&mut self, val: u32) {
        assert!(val > 0);
        self.escudos_in_circulation += val;
        self.produced_escudos += val;
    }

    pub fn remove(&mut self, val: u32) {
        assert!(val > 0);
        self.escudos_in_circulation -= val;
    } 
    
    pub fn reserve_escudos(&self) -> u32 {
        self.deposits * BASE_SHIP_ESCUDO_BALANCE
    }

    pub fn inflation_rate(&self) -> f32 {
        let reserve = self.reserve_escudos() as f32;
        (self.escudos_in_circulation as f32 - reserve) / reserve
    }

    pub fn payout(&mut self) -> u32 {
        let nominator = self.reserve_escudos() * BASE_ESCUDO_PAYOUT;
        let payout = nominator / self.escudos_in_circulation;
        self.produce(payout);
        payout
    }

    pub fn bonus_payout(&mut self, networth: u32) -> u32 {
        if networth <= 100 {
            return 0;
        }
        let surplus_networth = networth - BASE_SHIP_ESCUDO_BALANCE;
        (surplus_networth as f32 * NETWORTH_PAYOUT_PERCENTAGE) as u32

    }

    pub fn total_payout(&mut self, networth: u32) -> u32 {
        self.payout() + self.bonus_payout(networth)
    }
}
