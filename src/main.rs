extern crate image;
extern crate time;
extern crate rayon;
extern crate num;

use std::env;
use rayon::prelude::*;
use std::sync::atomic::{AtomicU64, Ordering};
use std::io;
use std::io::prelude::*;

fn main() {

	// Get the args
	let args: Vec<String> = env::args().collect();

	// Get the image
	if args.len() != 2 {
		println!("Usage: ./mandelbrot <width>");
		return;
	}

	let width = args[1].parse::<u32>().unwrap();
	let height = (width as f32*9.0/16.0) as u32;
	let iterations_per_pixel = (width as f64/25f64) as u32;

	println!("This size will take {:.6} GB of RAM, are you sure you want to do this?", (width as u64*height as u64*3u64) as f64/1000f64/1000f64/1000f64);
	let _ = io::stdin().read(&mut [0u8]).unwrap();

	let start = time::precise_time_s();
	
	let img = image::ImageBuffer::<image::Rgb<u8>, Vec<u8>>::new(width, height);

	// Distribute the work amongst the cores
	let mut buffer = img.clone().into_vec();

	// Status bar stuff
	let rows_done: AtomicU64 = AtomicU64::new(0_u64);

	// Split the image into rows
    buffer.par_chunks_mut(width as usize * 3usize)
        .enumerate()
        .for_each(|(y, row)| {
            // Iterate through all pixels in this row
            for x in 0..width {
				// Check if it's in the mandelbrot set
				match is_point_in_set(x, y, iterations_per_pixel, width, height) {
					Some(itr) => {
						let mut r_val = (itr as f32/iterations_per_pixel as f32)*255f32*2f32;
						if r_val > 255f32 {
							r_val = 255f32;
						}
						let mut gb_val = (1f32-(itr as f32/iterations_per_pixel as f32)) * -255f32 + 255f32;
						if gb_val < 0f32 {
							gb_val = 0f32;
						}
						row[x as usize * 3usize]		 = r_val as u8;
						row[x as usize * 3usize +1usize] = gb_val as u8;
						row[x as usize * 3usize +2usize] = gb_val as u8;
					},
					None => {
						row[x as usize * 3usize]		 = 255u8;
						row[x as usize * 3usize +1usize] = 255u8;
						row[x as usize * 3usize +2usize] = 255u8;
					}
				}
	        }
	        let rows_done_new = rows_done.fetch_add(1_u64, Ordering::Relaxed) + 1u64;
	        // Update the status bar
			let message = format!("{:>12} of {:>12} iterations done ({:>5.2}%). ETA: {:>05.1} seconds.\r", 
				rows_done_new*width as u64*width as u64/25u64,
				width as u64*height as u64*width as u64/25u64,
				((rows_done_new*width as u64*width as u64) as f64 / (width as u64*height as u64*width as u64) as f64)*100f64,
				((width as u64*height as u64*width as u64) as f64 - (rows_done_new*width as u64*width as u64) as f64) / ((rows_done_new*width as u64*width as u64) as f64 / (time::precise_time_s()-start))
			);
			print!("{:<}", message);
        });

    let final_image: image::RgbImage = image::ImageBuffer::from_vec(width, height, buffer).unwrap();

	// Save
	println!("\n\nGeneration finished. It took {} seconds", time::precise_time_s()-start);
	println!("Saving...");
	final_image.save("mandelbrot.png").unwrap();
	println!("{}x{} mandelbrot set image generated! It took {} seconds.", width, height, time::precise_time_s()-start);
}

fn is_point_in_set(x: u32, y: usize, iterations: u32, width: u32, height: u32) -> Option<u32> {
	let c = num::complex::Complex::new((x as f64/(width-1) as f64)*3.5f64-2.5f64, (y as f64/(height-1) as f64)*1.96875f64-0.984375f64);
	
	let mut z = num::complex::Complex::new(0f64, 0f64);

	for i in 0..iterations {
		z = z * z + c;
		if z.norm_sqr() > 4.0 {
			return Some(i);
		}
	}

	None
}