use argh::FromArgs;

pub fn parse() -> Args {
    argh::from_env()
}

#[derive(FromArgs, Debug)]
#[argh(description = "A program for downloading satellite imagery by geographic coordinates")]
pub struct Args {
    #[argh(
        option,
        short = 'p',
        description = "preferences file path",
        default = "String::from(\"./preferences.json\")"
    )]
    pub prefs: String,

    #[argh(
        switch,
        short = 'o',
        description = "open the result image in the default viewer"
    )]
    pub open: bool,

    #[argh(option, short = 'u', description = "URL template for the imagery")]
    pub url: Option<String>,

    #[argh(option, short = 's', description = "tile size in pixels")]
    pub tile_size: Option<u32>,

    #[argh(option, short = 'd', description = "out directory")]
    pub out_dir: Option<String>,

    #[argh(
        option,
        short = 't',
        description = "top left coordinates (52.70867508992417, 5.68805453553596)"
    )]
    pub top_left: Option<String>,

    #[argh(
        option,
        short = 'b',
        description = "bottom right coordinates (52.55494609768789, 5.879032918935248)"
    )]
    pub bottom_right: Option<String>,

    #[argh(option, short = 'z', description = "zoom level (recommended 13-18)")]
    pub zoom: Option<u8>,
}
