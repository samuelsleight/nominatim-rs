#![doc = include_str!("../README.md")]

use std::str::FromStr;

use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde::{Deserialize, Serialize};
use url::Url;

mod ident;

pub use ident::IdentificationMethod;

/// The interface for accessing a nominatim API server.
#[derive(Debug, Clone)]
pub struct Client {
    ident: IdentificationMethod, // how to access the server
    base_url: Url,               // defaults to https://nominatim.openstreetmap.org
    client: reqwest::Client,
}

impl Client {
    pub fn new(ident: IdentificationMethod) -> Self {
        Self {
            ident,
            base_url: Url::parse("https://nominatim.openstreetmap.org/").unwrap(),
            client: reqwest::Client::new(),
        }
    }

    /// Set the client's internal base url for all the requests.
    pub fn set_base_url(&mut self, url: impl AsRef<str>) -> Result<(), url::ParseError> {
        self.base_url = Url::parse(url.as_ref())?;

        Ok(())
    }

    /// Check the status of the nominatim server.
    pub async fn status(&self) -> Result<Status, reqwest::Error> {
        let mut url = self.base_url.clone();
        url.set_path("status.php");
        url.set_query(Some("format=json"));

        let mut headers = HeaderMap::new();
        headers.append(
            HeaderName::from_str(&self.ident.header()).unwrap(),
            HeaderValue::from_str(&self.ident.value()).unwrap(),
        );

        let response = match self.client.get(url).headers(headers).send().await {
            Ok(response) => response,
            Err(err) => return Err(err),
        };

        let status: Status = match response.json().await {
            Ok(status) => status,
            Err(err) => return Err(err),
        };

        Ok(status)
    }

    /// Get [`Place`]s from a search query.
    pub async fn search(&self, query: impl AsRef<str>) -> Result<Vec<Place>, reqwest::Error> {
        let mut url = self.base_url.clone();
        url.set_query(Some(&format!(
            "addressdetails=1&extratags=1&q={}&format=json",
            query.as_ref().replace(" ", "+")
        )));

        let mut headers = HeaderMap::new();
        headers.append(
            HeaderName::from_str(&self.ident.header()).unwrap(),
            HeaderValue::from_str(&self.ident.value()).unwrap(),
        );

        let response = match self.client.get(url).headers(headers).send().await {
            Ok(response) => response,
            Err(err) => return Err(err),
        };

        let places: Vec<Place> = match response.json().await {
            Ok(place) => place,
            Err(err) => return Err(err),
        };

        Ok(places)
    }

    /// Generate a [`Place`] from latitude and longitude
    pub async fn reverse(
        &self,
        latitude: impl AsRef<str>,
        longitude: impl AsRef<str>,
        zoom: Option<u8>,
    ) -> Result<Place, reqwest::Error> {
        let mut url = self.base_url.clone();
        url.set_path("/reverse");

        match zoom {
            Some(zoom) => {
                url.set_query(Some(&format!(
                    "addressdetails=1&extratags=1&format=json&lat={}&lon={}&zoom={}",
                    latitude.as_ref().replace(" ", ""),
                    longitude.as_ref().replace(" ", ""),
                    zoom
                )));
            }
            None => {
                url.set_query(Some(&format!(
                    "addressdetails=1&extratags=1&format=json&lat={}&lon={}",
                    latitude.as_ref().replace(" ", ""),
                    longitude.as_ref().replace(" ", ""),
                )));
            }
        }

        let mut headers = HeaderMap::new();
        headers.append(
            HeaderName::from_str(&self.ident.header()).unwrap(),
            HeaderValue::from_str(&self.ident.value()).unwrap(),
        );

        let response = match self.client.get(url).headers(headers).send().await {
            Ok(response) => response,
            Err(err) => return Err(err),
        };

        let place: Place = match response.json().await {
            Ok(place) => place,
            Err(err) => return Err(err),
        };

        Ok(place)
    }

    /// Return [`Place`]s from a list of OSM Node, Way, or Relations.
    pub async fn lookup(&self, queries: Vec<&str>) -> Result<Vec<Place>, reqwest::Error> {
        let queries = queries.join(",");

        let mut url = self.base_url.clone();
        url.set_path("/lookup");
        url.set_query(Some(&format!(
            "osm_ids={}&addressdetails=1&extratags=1&format=json",
            queries
        )));

        let mut headers = HeaderMap::new();
        headers.append(
            HeaderName::from_str(&self.ident.header()).unwrap(),
            HeaderValue::from_str(&self.ident.value()).unwrap(),
        );

        let response = match self.client.get(url).headers(headers).send().await {
            Ok(response) => response,
            Err(err) => return Err(err),
        };

        let places: Vec<Place> = match response.json().await {
            Ok(place) => place,
            Err(err) => return Err(err),
        };

        Ok(places)
    }
}

/// The status of a nominatim server.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Status {
    pub status: usize,
    pub message: String,
    pub data_updated: Option<String>,
    pub software_version: Option<String>,
    pub database_version: Option<String>,
}

/// A location returned by the nominatim server.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Place {
    #[serde(default)]
    pub place_id: usize,
    #[serde(default)]
    pub licence: String,
    #[serde(default)]
    pub osm_type: String,
    #[serde(default)]
    pub osm_id: usize,
    #[serde(default)]
    pub boundingbox: Vec<String>,
    #[serde(default)]
    pub lat: String,
    #[serde(default)]
    pub lon: String,
    #[serde(default)]
    pub display_name: String,
    pub class: Option<String>,
    #[serde(rename = "type")]
    pub _type: Option<String>,
    pub importance: Option<f64>,
    pub icon: Option<String>,
    #[serde(default)]
    pub address: Option<Address>,
    pub extratags: Option<ExtraTags>,
}

/// An address that a place can have.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Address {
    pub continent: Option<String>,

    pub country: Option<String>,
    pub country_code: Option<String>,

    pub region: Option<String>,
    pub state: Option<String>,
    pub state_district: Option<String>,
    pub county: Option<String>,

    #[serde(rename = "ISO3166-2-lvl4")]
    pub iso3166_2_lvl4: Option<String>,

    pub municipality: Option<String>,
    pub city: Option<String>,
    pub town: Option<String>,
    pub village: Option<String>,

    pub city_district: Option<String>,
    pub district: Option<String>,
    pub borough: Option<String>,
    pub suburb: Option<String>,
    pub subdivision: Option<String>,

    pub hamlet: Option<String>,
    pub croft: Option<String>,
    pub isolated_dwelling: Option<String>,

    pub neighbourhood: Option<String>,
    pub allotments: Option<String>,
    pub quarter: Option<String>,

    pub road: Option<String>,
    pub house_number: Option<String>,
    pub house_name: Option<String>,

    pub postcode: Option<String>,
}

/// Some extra metadata that a place might have.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExtraTags {
    pub capital: Option<String>,
    pub website: Option<String>,
    pub wikidata: Option<String>,
    pub wikipedia: Option<String>,
    pub population: Option<String>,
}
