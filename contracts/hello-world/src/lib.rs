#![no_std]

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, Address, Env, String};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Error {
    InvalidAmount = 1,
    DeductionExceedsDeposit = 2,
    RecordNotFound = 3,
    Unauthorized = 4,
    InvalidStatus = 5,
}

#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Status {
    Created = 0,
    DepositConfirmed = 1,
    Deducted = 2,
    RefundConfirmed = 3,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DepositRecord {
    pub id: u64,
    pub landlord: Address,
    pub student: Address,
    pub deposit: i128,
    pub deduction: i128,
    pub reason: String,
    pub status: Status,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    NextId,
    Record(u64),
}

#[contract]
pub struct RentDepositContract;

#[contractimpl]
impl RentDepositContract {
    // Tao ho so tien coc moi cho sinh vien va luu vao bo nho ben vung.
    pub fn create_deposit(
        env: Env,
        landlord: Address,
        student: Address,
        deposit: i128,
    ) -> Result<u64, Error> {
        landlord.require_auth();

        if deposit <= 0 {
            return Err(Error::InvalidAmount);
        }

        let id: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::NextId)
            .unwrap_or(1_u64);

        let record = DepositRecord {
            id,
            landlord,
            student,
            deposit,
            deduction: 0,
            reason: String::from_str(&env, ""),
            status: Status::Created,
        };

        env.storage().persistent().set(&DataKey::Record(id), &record);
        let next_id = id.checked_add(1).ok_or(Error::InvalidAmount)?;
        env.storage().persistent().set(&DataKey::NextId, &next_id);

        Ok(id)
    }

    // Sinh vien xac nhan da thanh toan tien coc cho chu nha.
    pub fn confirm_deposit(env: Env, student: Address, id: u64) -> Result<(), Error> {
        student.require_auth();

        let mut record: DepositRecord = env
            .storage()
            .persistent()
            .get(&DataKey::Record(id))
            .ok_or(Error::RecordNotFound)?;

        if record.student != student {
            return Err(Error::Unauthorized);
        }

        if record.status != Status::Created {
            return Err(Error::InvalidStatus);
        }

        record.status = Status::DepositConfirmed;
        env.storage().persistent().set(&DataKey::Record(id), &record);
        Ok(())
    }

    // Chu nha ghi nhan so tien khau tru va ly do khi sinh vien tra phong.
    pub fn record_deduction(
        env: Env,
        landlord: Address,
        id: u64,
        deduction: i128,
        reason: String,
    ) -> Result<(), Error> {
        landlord.require_auth();

        let mut record: DepositRecord = env
            .storage()
            .persistent()
            .get(&DataKey::Record(id))
            .ok_or(Error::RecordNotFound)?;

        if record.landlord != landlord {
            return Err(Error::Unauthorized);
        }

        if record.status != Status::DepositConfirmed {
            return Err(Error::InvalidStatus);
        }

        if deduction < 0 {
            return Err(Error::InvalidAmount);
        }

        if deduction > record.deposit {
            return Err(Error::DeductionExceedsDeposit);
        }

        record.deduction = deduction;
        record.reason = reason;
        record.status = Status::Deducted;

        env.storage().persistent().set(&DataKey::Record(id), &record);
        Ok(())
    }

    // Sinh vien xac nhan so tien hoan coc cuoi cung va tra ve so tien duoc hoan.
    pub fn confirm_refund(env: Env, student: Address, id: u64) -> Result<i128, Error> {
        student.require_auth();

        let mut record: DepositRecord = env
            .storage()
            .persistent()
            .get(&DataKey::Record(id))
            .ok_or(Error::RecordNotFound)?;

        if record.student != student {
            return Err(Error::Unauthorized);
        }

        if record.status != Status::Deducted {
            return Err(Error::InvalidStatus);
        }

        record.status = Status::RefundConfirmed;
        env.storage().persistent().set(&DataKey::Record(id), &record);

        let refund = record
            .deposit
            .checked_sub(record.deduction)
            .ok_or(Error::DeductionExceedsDeposit)?;
        Ok(refund)
    }

    // Doc va tra ve thong tin chi tiet cua mot ho so tien coc theo id.
    pub fn get_deposit(env: Env, id: u64) -> Result<DepositRecord, Error> {
        env.storage()
            .persistent()
            .get(&DataKey::Record(id))
            .ok_or(Error::RecordNotFound)
    }
}