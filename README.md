# lexoffice-invoice-uploader

Implements a voucher uploader to lexoffice in Rust.

## How to Use

Download [here](https://github.com/McTschecker/lexoffice-invoice-uploader/releases/latest) use gnu_win for windows 

Move executable to Invoices directory.

Start the programm, to upload all invoices included in the `invoices.csv` file



## Sample Config for Windows

```yaml
---
api_key: LEXOFFICEAPI_KEY
prefixes:
  - prefix: alias
    path: Alias
  - prefix: stockx
    path: StockX
customers:
  - customer_id: xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxx
    customer_adress: "GOAT, 3433 W Exposition Place, 90018, Los Angeles (CA), USA "
  - customer_id: xxxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxx
    customer_adress: "StockX LLC, 1046 Woodward Avenue, 48226, Detroit (MI), USA "
```
