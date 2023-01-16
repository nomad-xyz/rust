# Accessed Funds Calculator 

A quick and simple (and hopefully portable-ish) calculator to answer the question "how much has been accessed from the contract so far" 
## Requirements

- Needs a valid Etherscan API key to be configured in your local environment, or to be included with the run command
```
export ETHERSCAN_KEY=YOUR_KEY
```
- Requires a valid etherscan api url
```
export ETHERSCAN_API=https://api.etherscan.io/api
```
- Requires a valid token price api 
```
export PRICING_API=https://api.coingecko.com/api/v3/simple/price
```


## Usage
*Using the binary:*
```
./accessed_funds_calculator
```
*Using cargo:*
```
cargo run -p accessed-funds-calculator
```
