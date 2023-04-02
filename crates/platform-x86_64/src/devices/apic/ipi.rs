pub enum IpiDelivery {
    Fixed(u8),
    LowestPriority(u8),
    Smi,
    Nmi,
    Init,
    StartUp(u8),
}

impl IpiDelivery {
    const fn encode(&self) -> u32 {
        match self {
            IpiDelivery::Fixed(x) => 0xC4000 | (*x as u32),
            IpiDelivery::LowestPriority(x) => 0xC4100 | (*x as u32),
            IpiDelivery::Smi => 0xC4200,
            IpiDelivery::Nmi => 0xC4400,
            IpiDelivery::Init => 0xC4500,
            IpiDelivery::StartUp(x) => 0xC4600 | (*x as u32),
        }
    }
}

pub enum InterProcessorInterrupt {
    Physical {
        destination: u32,
        delivery: IpiDelivery,
    },
    Logical {
        destination: u32,
        delivery: IpiDelivery,
    },
    OnlySelf {
        vector: u8,
    },
    AllIncludingSelf {
        vector: u8,
    },
    AllExcludingSelf {
        delivery: IpiDelivery,
    },
}

impl InterProcessorInterrupt {
    pub const fn low(&self) -> u32 {
        match self {
            InterProcessorInterrupt::Physical { delivery, .. } => delivery.encode(),
            InterProcessorInterrupt::Logical { delivery, .. } => delivery.encode(),
            InterProcessorInterrupt::OnlySelf { vector } => 0x44000 | (*vector as u32),
            InterProcessorInterrupt::AllIncludingSelf { vector } => 0x84000 | (*vector as u32),
            InterProcessorInterrupt::AllExcludingSelf { delivery } => delivery.encode(),
        }
    }

    pub const fn high(&self) -> u32 {
        match self {
            InterProcessorInterrupt::Physical { destination, .. }
            | InterProcessorInterrupt::Logical { destination, .. } => *destination << 24,
            _ => 0,
        }
    }

    pub const fn encode(&self) -> u64 {
        let result = match self {
            InterProcessorInterrupt::Physical { destination, .. }
            | InterProcessorInterrupt::Logical { destination, .. } => (*destination as u64) << 32,
            _ => 0,
        };

        result | (self.low() as u64)
    }
}
