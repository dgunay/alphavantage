use crate::api::{APIRequest, APIRequestBuilder};
use crate::error::Error;
use crate::exchange_rate;
use crate::time_series;
use reqwest;
use std::io::Cursor;
use std::io::Read;

/// An asynchronous client for the Alpha Vantage API.
pub struct Client {
    builder: APIRequestBuilder,
    client: reqwest::Client,
}

impl Client {
    /// Create a new client using the specified API `key`.
    pub fn new(key: &str) -> Client {
        Client {
            builder: APIRequestBuilder::new(key),
            client: reqwest::Client::new(),
        }
    }

    /// Retrieve intraday time series for the specified `symbol` updated in realtime.
    pub async fn get_time_series_intraday(
        &self,
        symbol: &str,
        interval: time_series::IntradayInterval,
    ) -> Result<time_series::TimeSeries, Error> {
        self.get_time_series(&time_series::Function::IntraDay(interval), symbol)
            .await
    }

    /// Retrieve daily time series for the specified `symbol` including up to 20 years of historical data.
    pub async fn get_time_series_daily(
        &self,
        symbol: &str,
    ) -> Result<time_series::TimeSeries, Error> {
        self.get_time_series(&time_series::Function::Daily, symbol)
            .await
    }

    /// Retrieve weekly time series for the specified `symbol` including up to 20 years of historical data.
    pub async fn get_time_series_weekly(
        &self,
        symbol: &str,
    ) -> Result<time_series::TimeSeries, Error> {
        self.get_time_series(&time_series::Function::Weekly, symbol)
            .await
    }

    /// Retrieve monthly time series for the specified `symbol` including up to 20 years of historical data.
    pub async fn get_time_series_monthly(
        &self,
        symbol: &str,
    ) -> Result<time_series::TimeSeries, Error> {
        self.get_time_series(&time_series::Function::Monthly, symbol)
            .await
    }

    /// Retrieve the exchange rate from the currency specified by `from_currency_code` to the
    /// currency specified by `to_currency_code`.
    pub async fn get_exchange_rate(
        &self,
        from_currency_code: &str,
        to_currency_code: &str,
    ) -> Result<exchange_rate::ExchangeRate, Error> {
        let function = "CURRENCY_EXCHANGE_RATE";
        let params = vec![
            ("from_currency", from_currency_code),
            ("to_currency", to_currency_code),
        ];
        let request = self.builder.create(function, &params);
        let response = self.api_call(request).await?;
        let result = exchange_rate::parser::parse(response)?;
        Ok(result)
    }

    async fn get_time_series(
        &self,
        function: &time_series::Function,
        symbol: &str,
    ) -> Result<time_series::TimeSeries, Error> {
        let mut params = vec![("symbol", symbol)];
        if let time_series::Function::IntraDay(interval) = function {
            params.push(("interval", interval.to_string()));
        }
        let request = self.builder.create(function.into(), &params);
        let response = self.api_call(request).await?;
        let result = time_series::parser::parse(function, response)?;
        Ok(result)
    }

    async fn api_call<'a>(&self, request: APIRequest<'a>) -> Result<impl Read, Error> {
        let response = self.client.execute(request.into()).await?;
        let status = response.status();
        if status != reqwest::StatusCode::OK {
            return Err(Error::ServerError(status.as_u16()));
        }
        let reader = Cursor::new(response.bytes().await?);
        Ok(reader)
    }
}
