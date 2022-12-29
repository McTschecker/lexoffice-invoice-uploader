const BASE_URL: &'static str = "https://api.lexoffice.io/v1/";

pub mod invoice {
    use std::path::Path;
    use chrono::{NaiveDate};
    use log::{debug, error, info};
    use rust_decimal::Decimal;
    use serde::{Deserialize, Serialize};
    use crate::invoice::{german_date_format, BASE_URL};
    use crate::invoice::german_decimal_format;
    use crate::settings::{Config};
    use std::error;
    use reqwest::Client;
    use async_recursion::async_recursion;
    use reqwest::{multipart, Body};
    use tokio::fs::File;
    use tokio_util::codec::{BytesCodec, FramedRead};

    type Result<T> = std::result::Result<T, Box<dyn error::Error>>;
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
        currency: String,
        #[serde(rename = "Transaktionstyp")]
        transaction_type: String,
        #[serde(rename = "Rechnungsadresse")]
        billing_adress: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct CompletedInvoices {
        #[serde(rename = "Rechnungsnummer")]
        invoice_number: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct VoucherItem {
        amount: Decimal,
        #[serde(rename = "taxAmount")]
        tax_amount: Decimal,
        #[serde(rename = "taxRatePercent")]
        tax_rate_percent: Decimal,
        // Innergemeinschaftliche Lieferung
        #[serde(rename = "categoryId")]
        category_id: String
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct VoucherCreateRequest {
        #[serde(rename = "type")]
        type_of_voucher: String,
        #[serde(rename = "voucherNumber")]
        voucher_number: String,
        #[serde(rename = "voucherDate")]
        voucher_date: String,
        #[serde(rename = "shippingDate")]
        shipping_date: Option<String>,
        #[serde(rename = "dueDate")]
        due_date: Option<String>,
        #[serde(rename = "totalGrossAmount")]
        total_gross_amount: Decimal,
        #[serde(rename = "totalTaxAmount")]
        total_tax_amount: Decimal,
        // if b2b then net otherwise gross
        #[serde(rename = "taxType")]
        tax_type: String,
        #[serde(rename = "contactId")]
        contact_id: String,
        #[serde(rename = "voucherItems")]
        voucher_items: Vec<VoucherItem>,
    }

    #[derive(Deserialize, Debug)]
    struct VoucherCreationResponse {
        id: String,
        #[serde(rename = "resourceUri")]
        #[allow(dead_code)]
        resource_uri: String,
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct LexofficeError {
        message: String,
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
        pub fn get_invoice_number(&self) -> &String {
            match &self.internal_reference {
                Some(x) => x,
                None => &self.invoice_number
            }
        }

        pub fn validate(&self) -> bool {
            if self.currency != "EUR" {
                return false;
            }
            return true;
        }
        pub fn get_invoice_date_formatted(&self) -> String {
            self.invoice_date.format("%Y-%m-%d").to_string()
        }
        fn get_invoice_prefix(&self, config: &mut Config) -> Result<String> {
            // get the string before -
            let prefix_string = self.get_invoice_number().split("-").next().expect("No prefix found. The invoice number must have a prefix separated by a -");
            Ok(config.get_path(&prefix_string).expect("Error while getting prefix"))

        }
        fn get_shipping_date_formatted(&self) -> String {
            self.delivery_date.format("%Y-%m-%d").to_string()
        }

        #[async_recursion]
        pub async fn upload(&self, settings: &mut Config) -> Result<()>{
            let file_path = format!("{}/{}/{}.pdf", self.get_invoice_prefix(settings)?, self.invoice_date.format("%m-%Y"), self.invoice_number);
            let client = Client::new();
            // check if the file exists
            if !Path::new(&file_path).exists() {
                error!("File {} does not exist!", file_path);
                return Err("File does not exist!".into());
            }

            // check that the file is not empty and is smaller than 5 MB (limitation of lexofffice)
            let metadata = std::fs::metadata(&file_path)?;
            if metadata.len() == 0 || metadata.len() > 5_000_000 {
                error!("File {} is empty or too large!", file_path);
                return Err("File is empty or too large!".into());
            }

            // construct the upload request
            let upload_req = VoucherCreateRequest{
                type_of_voucher: "salesinvoice".to_string(),
                voucher_number: self.invoice_number.clone(),
                voucher_date: self.get_invoice_date_formatted(),
                shipping_date: Some(self.get_shipping_date_formatted()),
                due_date: None,
                total_gross_amount: self.final_amount,
                total_tax_amount: self.net-self.final_amount,
                tax_type: if self.transaction_type == "b2b".to_string() { "net".to_string() } else { "gross".to_string() },
                contact_id: settings.get_customer_id(&self.billing_adress)?,
                voucher_items: vec![VoucherItem{
                    amount: self.final_amount,
                    tax_amount: self.net-self.final_amount,
                    tax_rate_percent: self.vat,
                    category_id: "9075a4e3-66de-4795-a016-3889feca0d20".to_string(),
                }],
            };
            let res: reqwest::Response = client.post(format!("{}vouchers", BASE_URL))
                .bearer_auth(&settings.api_key)
                .json(&upload_req)
                .send()
                .await?;

            if res.status() == 401 {
                error!("API key is invalid! Please enter new one in the config file.");
                let error_message: LexofficeError = res.json().await?;
                error!("Error: {}", error_message.message);

                settings.invalidate_api_key();
                return self.upload(settings).await;
            }
            if res.status() != 200 {
                error!("Error while uploading invoice {}, got status code {}!", self.invoice_number, res.status());
                let error_message: LexofficeError = res.json().await?;
                error!("Error: {}", error_message.message);
                return Err("Error while uploading invoice!".into());
            }

            let result = res.json::<VoucherCreationResponse>().await?;
            info!("Successfully created voucher with id {}", result.id);

            debug!("Uploading file {} to voucher {}", file_path, result.id);

            let url = format!("{}vouchers/{}/files", BASE_URL, result.id);


            let file = File::open(&file_path).await?;

            // read file body stream
            let stream = FramedRead::new(file, BytesCodec::new());
            let file_body = Body::wrap_stream(stream);

            //make form part of file
            let some_file = multipart::Part::stream(file_body)
                .file_name("invoice.pdf")
                .mime_str("application/pdf")?;

            let form = multipart::Form::new()
                .text("type", "voucher")
                .part("file", some_file);

            let upload_res = client.post(&url)
                .bearer_auth(&settings.api_key)
                .multipart(form)
                .send()
                .await?;

            if upload_res.status() != 202 {
                error!("Error during file upload");
                return Err("Error during file upload".into());
            }

            info!("Successfully uploaded file {} to voucher {}", file_path, result.id);

            Ok(())
        }
        pub fn invoice_number(&self) -> &str {
            &self.invoice_number
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


