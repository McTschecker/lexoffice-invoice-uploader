
pub mod invoice {
    use chrono::{Datelike, NaiveDate};
    use log::error;
    use rust_decimal::Decimal;
    use serde::{Deserialize, Serialize};
    use crate::invoice::german_date_format;
    use crate::invoice::german_decimal_format;
    #[derive(Debug, Serialize, Deserialize)]
    pub struct InvoiceCSV {
        #[serde(rename = "Rechnungsnummer")]
        invoice_number: String,
        #[serde(rename = "Interne Referenz")]
        internal_reference: Option<String>,
        #[serde(rename = "Rechnungsdatum", with = "german_date_format")]
        invoice_date: NaiveDate,
        #[serde(rename = "Lieferdatum", with = "german_date_format")]
        delivery_date: NaiveDate,
        #[serde(rename = "Netto", with = "german_decimal_format")]
        net: Decimal,
        #[serde(rename = "USt. Rate (%)", with = "german_decimal_format")]
        vat: Decimal,
        #[serde(rename = "Endbetrag", with = "german_decimal_format")]
        final_amount: Decimal,
        #[serde(rename = "Währung")]
        currency: String
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct CompletedInvoices {
        #[serde(rename = "Rechnungsnummer")]
        invoice_number: String,
    }

    impl CompletedInvoices {

        pub fn invoice_number(&self) -> &str {
            &self.invoice_number
        }
        pub fn new(invoice: &InvoiceCSV) -> Self {
            Self { invoice_number: invoice.invoice_number.clone() }
        }
    }

    impl InvoiceCSV {
        pub fn validate(&self) -> bool {
            if self.currency != "EUR" {
                return false;
            }
            return true;
        }

        #[allow(dead_code)]
        fn get_date_path(&self) -> String {
            return format!("{}-{}/", self.invoice_date.month(), self.invoice_date.year());
        }
        #[allow(dead_code)]
        pub fn invoice_number(&self) -> &str {
            &self.invoice_number
        }
        #[allow(dead_code)]
        pub fn internal_reference(&self) -> &Option<String> {
            &self.internal_reference
        }
        #[allow(dead_code)]
        pub fn invoice_date(&self) -> NaiveDate {
            self.invoice_date
        }
        #[allow(dead_code)]
        pub fn delivery_date(&self) -> NaiveDate {
            self.delivery_date
        }
        #[allow(dead_code)]
        pub fn net(&self) -> Decimal {
            self.net
        }
        #[allow(dead_code)]
        pub fn vat(&self) -> Decimal {
            self.vat
        }
        #[allow(dead_code)]
        pub fn final_amount(&self) -> Decimal {
            self.final_amount
        }
        #[allow(dead_code)]
        pub fn currency(&self) -> &str {
            &self.currency
        }
    }

    pub fn read_invoice_csv(path: String) -> Vec<InvoiceCSV>{
        let mut rdr = csv::Reader::from_path(path).unwrap();
        let mut invoices: Vec<InvoiceCSV> = Vec::new();
        for result in rdr.deserialize() {
            if result.is_err() {
                error!("Error parsing invoice: {}", result.unwrap_err());
                continue;
            }
            let record: InvoiceCSV = result.unwrap();
            if record.validate() {
                invoices.push(record);
            }else {
                error!("invoice {} is not valid", record.invoice_number);
            }
        }
        invoices
    }

    fn read_done_invoice_csv(path: String) -> Vec<CompletedInvoices>{
        let mut rdr = csv::Reader::from_path(path).unwrap();
        let mut invoices: Vec<CompletedInvoices> = Vec::new();
        for result in rdr.deserialize() {
            let record: CompletedInvoices = result.unwrap();
            invoices.push(record);
        }
        invoices
    }

    pub fn try_read_done_invoices_csv(path: String) -> Vec<CompletedInvoices>{
        // verify if the file exists
        if std::path::Path::new(&path).exists() {
            return read_done_invoice_csv(path);
        }

        return vec![];
    }

    pub fn write_done_invoice_csv(path: &String, invoices: &Vec<CompletedInvoices>) {
        let mut wtr = csv::Writer::from_path(path).expect(&*format!("Could not open file {}", path));
        for invoice in invoices {
            wtr.serialize(invoice).expect("Could not serialize written invoice")
        }
        wtr.flush().expect("Could not flush written invoice");
    }
}

mod german_date_format {
    use chrono::{ NaiveDate};
    use serde::{self, Deserialize, Serializer, Deserializer};

    const FORMAT: &'static str = "%d.%m.%Y";

    // The signature of a serialize_with function must follow the pattern:
    //
    //    fn serialize<S>(&T, S) -> Result<S::Ok, S::Error>
    //    where
    //        S: Serializer
    //
    // although it may also be generic over the input types T.
    pub fn serialize<S>(
        date: &NaiveDate,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        let s = format!("{}", date.format(FORMAT));
        serializer.serialize_str(&s)
    }

    // The signature of a deserialize_with function must follow the pattern:
    //
    //    fn deserialize<'de, D>(D) -> Result<T, D::Error>
    //    where
    //        D: Deserializer<'de>
    //
    // although it may also be generic over the output types T.
    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<NaiveDate, D::Error>
        where
            D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        NaiveDate::parse_from_str(&*s, FORMAT).map_err(serde::de::Error::custom)
    }
}

mod german_decimal_format {
    use std::str::FromStr;
    use rust_decimal::Decimal;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(
        decimal: &rust_decimal::Decimal,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        let s = decimal.to_string();
        serializer.serialize_str(&*s)
    }

    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<Decimal, D::Error>
        where
            D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?.replace(",", ".");
        Decimal::from_str(&*s).map_err(serde::de::Error::custom)
    }
}


