use std::f32::consts::FRAC_PI_2;

use cgmath::{Angle, Euler, Point3, Rad, SquareMatrix, Vector3, Vector4};
use winit::{event::KeyEvent, keyboard::{KeyCode, PhysicalKey}};


#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

pub struct Camera {
    pub dir: Euler<cgmath::Deg<f32>>,
    pub pos: Point3<f32>,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

impl Camera {
    fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        use cgmath::InnerSpace;
        let (yaw_sin, yaw_cos) = self.dir.y.0.sin_cos();
        let (pitch_sin, pitch_cos) = self.dir.x.0.sin_cos();
        let forward = Vector3::new(pitch_cos * yaw_sin, -pitch_sin, pitch_cos * yaw_cos).normalize();

        let view = cgmath::Matrix4::look_to_rh(self.pos, forward, Vector3::new(0.0, 1.0, 0.0));
        let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);
        proj * view
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        use cgmath::SquareMatrix;
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    pub fn update_view_proj(&mut self, camera: &Camera) {
        self.view_proj = (OPENGL_TO_WGPU_MATRIX * camera.build_view_projection_matrix()).into();
    }
}

pub struct CameraController {
    speed: f32,
    is_up_pressed: bool,
    is_down_pressed: bool,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
}

impl CameraController {
    pub fn new(speed: f32) -> Self {
        Self {
            speed,
            is_up_pressed: false,
            is_down_pressed: false,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
        }
    }

    pub fn process_events(&mut self, event: &crate::WindowEvent) -> bool {
        match event {
            crate::WindowEvent::KeyboardInput {
                event: KeyEvent {
                    physical_key: PhysicalKey::Code(keycode),
                    state,
                    ..
                },
                ..
            } => {
                let is_pressed = state.is_pressed();
                match keycode {
                    KeyCode::Space => {
                        self.is_up_pressed = is_pressed;
                        true
                    }
                    KeyCode::ShiftLeft => {
                        self.is_down_pressed = is_pressed;
                        true
                    }
                    KeyCode::KeyW | KeyCode::ArrowUp => {
                        self.is_forward_pressed = is_pressed;
                        true
                    }
                    KeyCode::KeyA | KeyCode::ArrowLeft => {
                        self.is_left_pressed = is_pressed;
                        true
                    }
                    KeyCode::KeyS | KeyCode::ArrowDown => {
                        self.is_backward_pressed = is_pressed;
                        true
                    }
                    KeyCode::KeyD | KeyCode::ArrowRight => {
                        self.is_right_pressed = is_pressed;
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }

    pub fn update_camera(&self, camera: &mut Camera) {
        use cgmath::InnerSpace;
        let (yaw_sin, yaw_cos) = camera.dir.y.0.sin_cos();
        let (pitch_sin, pitch_cos) = camera.dir.x.0.sin_cos();
        //let forward = Vector3::new(yaw_sin, 0.0, yaw_cos).normalize();
        //let right = Vector3::new(-yaw_cos, 0.0, yaw_sin).normalize();
        let forward = Vector3::new(pitch_cos * yaw_sin, -pitch_sin, pitch_cos * yaw_cos).normalize();
        let right = Vector3::new(yaw_cos, 0.0, -yaw_sin).normalize();
        let up = forward.cross(right);

        if self.is_forward_pressed {
            println!("Camera dir is {:?}", camera.dir);
            println!("Foward is {:?}", forward);
            camera.pos += forward * self.speed;
            //println!("Now at {:?}", camera.pos);
        } else if self.is_backward_pressed {
            //println!("Foward is {:?}", forward);
            println!("Foward is {:?}", forward);
            camera.pos -= forward * self.speed;
            //println!("Now at {:?}", camera.pos);
        } else if self.is_right_pressed {
            println!("Right is {:?}", right);
            camera.pos += right * self.speed;
        } else if self.is_left_pressed {
            println!("Right is {:?}", right);
            camera.pos -= right * self.speed;
        }

        if self.is_up_pressed {
            camera.pos += up * self.speed;
        } else if self.is_down_pressed {
            camera.pos -= up * self.speed;
        }

        /*use cgmath::InnerSpace;
        let forward = camera.target - camera.eye;
        let forward_norm = forward.normalize();
        let forward_mag = forward.magnitude();

        // Prevents glitching when camera gets too close to the
        // center of the scene.
        if self.is_forward_pressed && forward_mag > self.speed {
            camera.eye += forward_norm * self.speed;
        }
        if self.is_backward_pressed {
            camera.eye -= forward_norm * self.speed;
        }

        let right = forward_norm.cross(camera.up);

        // Redo radius calc in case the up/ down is pressed.
        let forward = camera.target - camera.eye;
        let forward_mag = forward.magnitude();

        if self.is_right_pressed {
            // Rescale the distance between the target and eye so
            // that it doesn't change. The eye therefore still
            // lies on the circle made by the target and eye.
            camera.eye = camera.target - (forward + right * self.speed).normalize() * forward_mag;
        }
        if self.is_left_pressed {
            camera.eye = camera.target - (forward - right * self.speed).normalize() * forward_mag;
        }*/
    }
}