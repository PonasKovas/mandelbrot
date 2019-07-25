#![feature(atomic_min_max)]

extern crate image;
extern crate time;
extern crate rayon;
extern crate num;
extern crate minifb;

use rayon::prelude::*;
use std::sync::atomic::{AtomicU32, Ordering};

const ZOOM_POWER: f64 = 3.0;

struct Zoom {
	center: (f64, f64),
	zoom: f64
}

fn main() {

	let mut width = 1024;

	let args: Vec<String> = std::env::args().collect();

	if args.len() == 2 {
		width = args[1].parse::<usize>().expect("Could not parse the window width parameter.")
	}
	if args.len() > 2 {
		println!("Too many arguments!");
		return;
	}
	let height = (width as f64*2.4/3.5) as usize;

	let mut window = match minifb::Window::new("Mandelbrot", width, height, minifb::WindowOptions::default()) {
		Ok(win) => win,
		Err(err) => {
			println!("Unable to create window {}", err);
			return;
		}
	};
	window.set_cursor_style(minifb::CursorStyle::Crosshair);

	let mut zoom = Zoom{center: (-0.25, 0.), zoom: 1.0};

	let mut image_buffer: Vec<u32> = vec![0u32; width*height];

	generate_image(width, height, &zoom, &mut image_buffer);

	let mut left_clicked = false;
	let mut right_clicked = false;

	while window.is_open() {
		window.update_with_buffer(&image_buffer[..]);

		// Left click - zoom
		let left_pressed = window.get_mouse_down(minifb::MouseButton::Left);
		if left_pressed && !left_clicked {
			left_clicked = true;

			let click_pos = window.get_mouse_pos(minifb::MouseMode::Clamp);
			match click_pos {
				Some(pos) => {
					// Zoom

					let old_zoom = Zoom{center: zoom.center, zoom: zoom.zoom};

					zoom.zoom *= ZOOM_POWER;
					zoom.center = (
						(old_zoom.center.0 - 2.0/old_zoom.zoom) + (pos.0 as f64/(width-1) as f64) * 3.5/old_zoom.zoom,
						(old_zoom.center.1 - 1.2/old_zoom.zoom) + (pos.1 as f64/(height-1) as f64) * 2.4/old_zoom.zoom
					);
					generate_image(width, height, &zoom, &mut image_buffer);
				},
				None => ()
			}
		}
		if !left_pressed && left_clicked { left_clicked = false;}

		// Right click - unzoom
		let right_pressed = window.get_mouse_down(minifb::MouseButton::Right);
		if right_pressed && !right_clicked {
			right_clicked = true;

			let click_pos = window.get_mouse_pos(minifb::MouseMode::Clamp);
			match click_pos {
				Some(pos) => {
					// Unzoom

					let old_zoom = Zoom{center: zoom.center, zoom: zoom.zoom};

					zoom.zoom /= ZOOM_POWER;
					zoom.center = (
						(old_zoom.center.0 - 2.0/old_zoom.zoom) + (((width-1) as f64-pos.0 as f64)/(width-1) as f64) * 3.5/old_zoom.zoom,
						(old_zoom.center.1 - 1.2/old_zoom.zoom) + (((height-1) as f64-pos.1 as f64)/(height-1) as f64) * 2.4/old_zoom.zoom
					);

					generate_image(width, height, &zoom, &mut image_buffer);
				},
				None => ()
			}
		}
		if !right_pressed && right_clicked { right_clicked = false;}

	}



}

fn generate_image(width: usize, height: usize, zoom: &Zoom, buffer: &mut Vec<u32>) {

	let iterations_per_pixel = width as u32;

	let mut escape = vec![iterations_per_pixel; width*height];

	let lowest = AtomicU32::new(iterations_per_pixel as u32);
	let highest = AtomicU32::new(0u32);
	
	// Get the escapes
    escape.par_chunks_mut(width as usize)
        .enumerate()
        .for_each(|(y, row)| {
            // Iterate through all pixels in this row
            for x in 0..width {
				// Check if it's in the mandelbrot set
				let real: f64 = (zoom.center.0 - 2.0/zoom.zoom) + (x as f64/(width-1) as f64) * 3.5/zoom.zoom;
				let imaginary: f64 = (zoom.center.1 - 1.2/zoom.zoom) + (y as f64/(height-1) as f64) * 2.4/zoom.zoom;

				match is_point_in_set(real, imaginary, iterations_per_pixel) {
					Some(itr) => {
						row[x as usize] = itr;
						lowest.fetch_min(itr as u32, Ordering::Relaxed);
						highest.fetch_max(itr as u32, Ordering::Relaxed);
					},
					None => {
						row[x as usize] = iterations_per_pixel as u32;
						highest.fetch_max(iterations_per_pixel as u32, Ordering::Relaxed);
					}
				}
	        }
        });

    let h = highest.load(Ordering::Relaxed);
    let l = lowest.load(Ordering::Relaxed);

    // Now add colors depending on the highest, lowest escapes
    buffer.par_chunks_mut(width as usize)
	    .enumerate()
	    .for_each(|(y, row)| {
	        // Iterate through all pixels in this row
	        for x in 0..width {
	        	let red = 255f64 *
	        		(escape[(y*width+x) as usize] - l) as f64 /
	        		(h - l) as f64;
				let mut greenblue = 2.0*255f64 *
					(escape[(y*width+x) as usize] - l) as f64 /
					(h - l) as f64;
				if greenblue > 255f64 {
					greenblue = 255f64;
				}

				row[x as usize] = (255u32 << 24) |
					((red as u32) << 16) |
					((greenblue as u32) << 8) |
					greenblue as u32;
	        }
    	});
}

fn is_point_in_set(x: f64, y: f64, iterations: u32) -> Option<u32> {
	let c = num::complex::Complex::new(x, y);
	
	let mut z = num::complex::Complex::new(0f64, 0f64);

	for i in 0..iterations {
		z = z * z + c;
		if z.norm_sqr() > 4.0 {
			return Some(i);
		}
	}

	None
}