use anyhow::Result;

mod simple_detector {
    use anyhow::{ensure, Context, Error, Result};
    use apriltag::{DetectorBuilder, Family, TagParams};
    use mat2image::ToImage;
    use opencv::core::Mat;
    use opencv::core::VecN;
    use opencv::prelude::MatTraitConstManual;
    use opencv::prelude::VideoCaptureTrait;
    use opencv::prelude::VideoCaptureTraitConst;
    use opencv::{core::Rect, core::Scalar, highgui, imgproc, videoio};
    use std::{path::PathBuf, str::FromStr};
    use std::{thread, time::Duration, time::Instant};

    #[derive(Debug, Clone)]
    struct TagParamsArg {
        pub tagsize: f64,
        pub fx: f64,
        pub fy: f64,
        pub cx: f64,
        pub cy: f64,
    }

    impl From<TagParamsArg> for TagParams {
        fn from(arg: TagParamsArg) -> Self {
            let TagParamsArg {
                tagsize,
                fx,
                fy,
                cx,
                cy,
            } = arg;

            Self {
                tagsize,
                fx,
                fy,
                cx,
                cy,
            }
        }
    }

    impl FromStr for TagParamsArg {
        type Err = Error;

        fn from_str(text: &str) -> Result<Self, Self::Err> {
            let tokens: Vec<_> = text.split(',').collect();
            ensure!(
                tokens.len() == 5,
                r#"tag parameters must be in format "tagsize,fx,fy,cx,cy""#
            );

            let values = tokens
                .into_iter()
                .map(|token| -> Result<_> {
                    let value: f64 = token.parse().unwrap();
                    Ok(value)
                })
                .collect::<Result<Vec<_>>>()
                .with_context(|| format!("failed to parse tag parameters {}", text))?;

            Ok(Self {
                tagsize: values[0],
                fx: values[1],
                fy: values[2],
                cx: values[3],
                cy: values[4],
            })
        }
    }

    pub fn _main() -> Result<()> {
        let mut cam = videoio::VideoCapture::new(0, videoio::CAP_ANY)?;
        let res = (1280.0, 720.0);
        highgui::named_window("seancv", 1)?;
        cam.set(3, res.0)?;
        cam.set(4, res.1)?;
        let _ = videoio::VideoCapture::is_opened(&cam)?;
        let family: Family = Family::tag_16h5();
        // let tag_params: Option<TagParams> = tag_params.map(|params| params.into());
        let mut detector = DetectorBuilder::new()
            .add_family_bits(family, 1)
            .build()
            .unwrap();

        let mut start = Instant::now();
        let mut frame_num = 0;
        let mut first = true;
        loop {
            if frame_num % 10 == 0 {
                start = Instant::now();
                frame_num = 0;
            }
            frame_num += 1;
            let mut frame = Mat::default();
            cam.read(&mut frame)?;
            if frame.size()?.width == 0 || frame.size()?.height == 0 {
                thread::sleep(Duration::from_millis(50));
            }

            // if input_files.is_empty() {
            //     eprintln!("no input files");
            //     return Ok(());
            // }

            let frame_img = frame.to_image().unwrap();
            // let DynamicImage::ImageLuma8(frame) = frame else {
            //     unreachable!();
            // };
            let detections = detector.detect(frame_img.to_luma8());
            // let detections = detector.detect(frame.to_image().unwrap().as_flat_samples_u8().unwrap());

            if first {
                start = Instant::now();
                first = false;
            }
            detections.into_iter().enumerate().for_each(|(index, det)| {
                println!("  - detection {}: {:#?}", index, det);
                println!("DISTANCE {:#?}", det.hamming());
                println!("fps {:?}", frame_num as f32 / start.elapsed().as_secs_f32());
                let pose = det.estimate_tag_pose(&TagParams {
                    tagsize: 1.00,
                    fx: 2.1,
                    fy: 2.2,
                    cx: 4.0,
                    cy: 5.0,
                });
                println!("  - pose {}: {:#?}", index, pose);
                if let Some(pose) = pose {
                    println!("translation {:?}", pose.translation());
                    println!("rotation {:?}", pose.rotation());
                }
                let corners = det.corners();
                let center = det.center();
                let (w, h) = (
                    (corners[2][0] - corners[3][0]) as i32,
                    (corners[0][1] - corners[3][1]) as i32,
                );
                let r#box = Rect {
                    x: center[0] as i32 - w / 2,
                    y: center[1] as i32 - h / 2,
                    width: w,
                    height: h,
                };
                imgproc::rectangle(
                    &mut frame,
                    r#box,
                    Scalar::new(255f64, 0f64, 0f64, 25f64), // color value
                    1,                                      // border width of drawn rectangle
                    8,
                    0,
                )
                .unwrap();
                imgproc::line(
                    &mut frame,
                    opencv::core::Point_::new(corners[0][0] as i32, corners[0][1] as i32),
                    opencv::core::Point_::new(corners[1][0] as i32, corners[1][1] as i32),
                    Scalar::new(255f64, 255f64, 0f64, 0f64), // color value
                    4,
                    2,
                    0,
                )
                .unwrap();
                imgproc::line(
                    &mut frame,
                    opencv::core::Point_::new(corners[0][0] as i32, corners[0][1] as i32),
                    opencv::core::Point_::new(corners[3][0] as i32, corners[3][1] as i32),
                    Scalar::new(255f64, 255f64, 0f64, 0f64), // color value
                    4,
                    2,
                    0,
                )
                .unwrap();
                imgproc::line(
                    &mut frame,
                    opencv::core::Point_::new(corners[3][0] as i32, corners[3][1] as i32),
                    opencv::core::Point_::new(corners[2][0] as i32, corners[2][1] as i32),
                    Scalar::new(255f64, 255f64, 0f64, 0f64), // color value
                    4,
                    2,
                    0,
                )
                .unwrap();
                imgproc::line(
                    &mut frame,
                    opencv::core::Point_::new(corners[2][0] as i32, corners[2][1] as i32),
                    opencv::core::Point_::new(corners[1][0] as i32, corners[1][1] as i32),
                    Scalar::new(255f64, 255f64, 0f64, 0f64), // color value
                    4,
                    2,
                    0,
                )
                .unwrap();
                imgproc::circle(
                    &mut frame,
                    opencv::core::Point_::new(center[0] as i32, center[1] as i32),
                    5,
                    Scalar::new(255f64, 255f64, 0f64, 0f64),
                    2,
                    2,
                    0,
                )
                .unwrap();
            });
            highgui::imshow("seancv", &frame)?;
            if highgui::wait_key(1)? > 0 {
                continue;
            }
        }
    }
}

fn main() -> Result<()> {
    simple_detector::_main()
}
