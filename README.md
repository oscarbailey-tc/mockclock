# Mock Clock

Tiny program that can be used to mock the clock sysvar. Program stores a u64 value in an account.

If you need to mock the clock sysvar, create a function that reads this account instead of the sysvar depending on build
env.
