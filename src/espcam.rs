use std::marker::PhantomData;

use esp_idf_hal::gpio::*;
use esp_idf_hal::peripheral::Peripheral;
use esp_idf_sys::*;

pub struct FrameBuffer<'a> {
    fb: *mut camera::camera_fb_t,
    _p: PhantomData<&'a camera::camera_fb_t>,
}

impl<'a> FrameBuffer<'a> {
    pub fn data(&self) -> &'a [u8] {
        unsafe { std::slice::from_raw_parts((*self.fb).buf, (*self.fb).len) }
    }

    pub fn width(&self) -> usize {
        unsafe { (*self.fb).width }
    }

    pub fn height(&self) -> usize {
        unsafe { (*self.fb).height }
    }

    pub fn format(&self) -> camera::pixformat_t {
        unsafe { (*self.fb).format }
    }

    pub fn timestamp(&self) -> camera::timeval {
        unsafe { (*self.fb).timestamp }
    }

    pub fn fb_return(&self) {
        unsafe { camera::esp_camera_fb_return(self.fb) }
    }
}

impl Drop for FrameBuffer<'_> {
    fn drop(&mut self) {
        self.fb_return();
    }
}

pub struct CameraSensor<'a> {
    sensor: *mut camera::sensor_t,
    _p: PhantomData<&'a camera::sensor_t>,
}

impl<'a> CameraSensor<'a> {
    pub fn init_status(&self) -> Result<(), EspError> {
        esp!(unsafe { (*self.sensor).init_status.unwrap()(self.sensor) })
    }
    pub fn reset(&self) -> Result<(), EspError> {
        esp!(unsafe { (*self.sensor).reset.unwrap()(self.sensor) })
    }
    pub fn set_pixformat(&self, format: camera::pixformat_t) -> Result<(), EspError> {
        esp!(unsafe { (*self.sensor).set_pixformat.unwrap()(self.sensor, format) })
    }
    pub fn set_framesize(&self, framesize: camera::framesize_t) -> Result<(), EspError> {
        esp!(unsafe { (*self.sensor).set_framesize.unwrap()(self.sensor, framesize) })
    }
    pub fn set_contrast(&self, level: i32) -> Result<(), EspError> {
        esp!(unsafe { (*self.sensor).set_contrast.unwrap()(self.sensor, level) })
    }
    pub fn set_brightness(&self, level: i32) -> Result<(), EspError> {
        esp!(unsafe { (*self.sensor).set_brightness.unwrap()(self.sensor, level) })
    }
    pub fn set_saturation(&self, level: i32) -> Result<(), EspError> {
        esp!(unsafe { (*self.sensor).set_saturation.unwrap()(self.sensor, level) })
    }
    pub fn set_sharpness(&self, level: i32) -> Result<(), EspError> {
        esp!(unsafe { (*self.sensor).set_sharpness.unwrap()(self.sensor, level) })
    }
    pub fn set_denoise(&self, level: i32) -> Result<(), EspError> {
        esp!(unsafe { (*self.sensor).set_denoise.unwrap()(self.sensor, level) })
    }
    pub fn set_gainceiling(&self, gainceiling: camera::gainceiling_t) -> Result<(), EspError> {
        esp!(unsafe { (*self.sensor).set_gainceiling.unwrap()(self.sensor, gainceiling) })
    }
    pub fn set_quality(&self, quality: i32) -> Result<(), EspError> {
        esp!(unsafe { (*self.sensor).set_quality.unwrap()(self.sensor, quality) })
    }
    pub fn set_colorbar(&self, enable: bool) -> Result<(), EspError> {
        esp!(unsafe {
            (*self.sensor).set_colorbar.unwrap()(self.sensor, if enable { 1 } else { 0 })
        })
    }
    pub fn set_whitebal(&self, enable: bool) -> Result<(), EspError> {
        esp!(unsafe {
            (*self.sensor).set_whitebal.unwrap()(self.sensor, if enable { 1 } else { 0 })
        })
    }
    pub fn set_gain_ctrl(&self, enable: bool) -> Result<(), EspError> {
        esp!(unsafe {
            (*self.sensor).set_gain_ctrl.unwrap()(self.sensor, if enable { 1 } else { 0 })
        })
    }
    pub fn set_exposure_ctrl(&self, enable: bool) -> Result<(), EspError> {
        esp!(unsafe {
            (*self.sensor).set_exposure_ctrl.unwrap()(self.sensor, if enable { 1 } else { 0 })
        })
    }
    pub fn set_hmirror(&self, enable: bool) -> Result<(), EspError> {
        esp!(unsafe {
            (*self.sensor).set_hmirror.unwrap()(self.sensor, if enable { 1 } else { 0 })
        })
    }
    pub fn set_vflip(&self, enable: bool) -> Result<(), EspError> {
        esp!(unsafe { (*self.sensor).set_vflip.unwrap()(self.sensor, if enable { 1 } else { 0 }) })
    }
    pub fn set_aec2(&self, enable: bool) -> Result<(), EspError> {
        esp!(unsafe { (*self.sensor).set_aec2.unwrap()(self.sensor, if enable { 1 } else { 0 }) })
    }
    pub fn set_awb_gain(&self, enable: bool) -> Result<(), EspError> {
        esp!(unsafe {
            (*self.sensor).set_awb_gain.unwrap()(self.sensor, if enable { 1 } else { 0 })
        })
    }
    pub fn set_agc_gain(&self, gain: i32) -> Result<(), EspError> {
        esp!(unsafe { (*self.sensor).set_agc_gain.unwrap()(self.sensor, gain) })
    }
    pub fn set_aec_value(&self, gain: i32) -> Result<(), EspError> {
        esp!(unsafe { (*self.sensor).set_aec_value.unwrap()(self.sensor, gain) })
    }
    pub fn set_special_effect(&self, effect: i32) -> Result<(), EspError> {
        esp!(unsafe { (*self.sensor).set_special_effect.unwrap()(self.sensor, effect) })
    }
    pub fn set_wb_mode(&self, mode: i32) -> Result<(), EspError> {
        esp!(unsafe { (*self.sensor).set_wb_mode.unwrap()(self.sensor, mode) })
    }
    pub fn set_ae_level(&self, level: i32) -> Result<(), EspError> {
        esp!(unsafe { (*self.sensor).set_ae_level.unwrap()(self.sensor, level) })
    }
    pub fn set_dcw(&self, enable: bool) -> Result<(), EspError> {
        esp!(unsafe { (*self.sensor).set_dcw.unwrap()(self.sensor, if enable { 1 } else { 0 }) })
    }
    pub fn set_bpc(&self, enable: bool) -> Result<(), EspError> {
        esp!(unsafe { (*self.sensor).set_bpc.unwrap()(self.sensor, if enable { 1 } else { 0 }) })
    }
    pub fn set_wpc(&self, enable: bool) -> Result<(), EspError> {
        esp!(unsafe { (*self.sensor).set_wpc.unwrap()(self.sensor, if enable { 1 } else { 0 }) })
    }
    pub fn set_raw_gma(&self, enable: bool) -> Result<(), EspError> {
        esp!(unsafe {
            (*self.sensor).set_raw_gma.unwrap()(self.sensor, if enable { 1 } else { 0 })
        })
    }
    pub fn set_lenc(&self, enable: bool) -> Result<(), EspError> {
        esp!(unsafe { (*self.sensor).set_lenc.unwrap()(self.sensor, if enable { 1 } else { 0 }) })
    }
    pub fn get_reg(&self, reg: i32, mask: i32) -> Result<(), EspError> {
        esp!(unsafe { (*self.sensor).get_reg.unwrap()(self.sensor, reg, mask) })
    }
    pub fn set_reg(&self, reg: i32, mask: i32, value: i32) -> Result<(), EspError> {
        esp!(unsafe { (*self.sensor).set_reg.unwrap()(self.sensor, reg, mask, value) })
    }
    pub fn set_res_raw(
        &self,
        start_x: i32,
        start_y: i32,
        end_x: i32,
        end_y: i32,
        offset_x: i32,
        offset_y: i32,
        total_x: i32,
        total_y: i32,
        output_x: i32,
        output_y: i32,
        scale: bool,
        binning: bool,
    ) -> Result<(), EspError> {
        esp!(unsafe {
            (*self.sensor).set_res_raw.unwrap()(
                self.sensor,
                start_x,
                start_y,
                end_x,
                end_y,
                offset_x,
                offset_y,
                total_x,
                total_y,
                output_x,
                output_y,
                scale,
                binning,
            )
        })
    }
    pub fn set_pll(
        &self,
        bypass: i32,
        mul: i32,
        sys: i32,
        root: i32,
        pre: i32,
        seld5: i32,
        pclken: i32,
        pclk: i32,
    ) -> Result<(), EspError> {
        esp!(unsafe {
            (*self.sensor).set_pll.unwrap()(
                self.sensor,
                bypass,
                mul,
                sys,
                root,
                pre,
                seld5,
                pclken,
                pclk,
            )
        })
    }
    pub fn set_xclk(&self, timer: i32, xclk: i32) -> Result<(), EspError> {
        esp!(unsafe { (*self.sensor).set_xclk.unwrap()(self.sensor, timer, xclk) })
    }
}

pub struct Camera<'a> {
    _p: PhantomData<&'a ()>,
}

impl<'a> Camera<'a> {
    pub fn new(
        pin_pwdn: impl Peripheral<P = impl InputPin + OutputPin> + 'a,
        pin_xclk: impl Peripheral<P = impl InputPin + OutputPin> + 'a,
        pin_d0: impl Peripheral<P = impl InputPin + OutputPin> + 'a,
        pin_d1: impl Peripheral<P = impl InputPin + OutputPin> + 'a,
        pin_d2: impl Peripheral<P = impl InputPin + OutputPin> + 'a,
        pin_d3: impl Peripheral<P = impl InputPin + OutputPin> + 'a,
        pin_d4: impl Peripheral<P = impl InputPin> + 'a,
        pin_d5: impl Peripheral<P = impl InputPin> + 'a,
        pin_d6: impl Peripheral<P = impl InputPin> + 'a,
        pin_d7: impl Peripheral<P = impl InputPin> + 'a,
        pin_vsync: impl Peripheral<P = impl InputPin + OutputPin> + 'a,
        pin_href: impl Peripheral<P = impl InputPin + OutputPin> + 'a,
        pin_pclk: impl Peripheral<P = impl InputPin + OutputPin> + 'a,
        pin_sda: impl Peripheral<P = impl InputPin + OutputPin> + 'a,
        pin_scl: impl Peripheral<P = impl InputPin + OutputPin> + 'a,
        pixel_format: camera::pixformat_t,
        frame_size: camera::framesize_t,
    ) -> Result<Self, esp_idf_sys::EspError> {
        esp_idf_hal::into_ref!(
            pin_pwdn, pin_xclk, pin_d0, pin_d1, pin_d2, pin_d3, pin_d4, pin_d5, pin_d6, pin_d7,
            pin_vsync, pin_href, pin_pclk, pin_sda, pin_scl
        );
        let config = camera::camera_config_t {
            pin_pwdn: pin_pwdn.pin(),
            pin_xclk: pin_xclk.pin(),
            pin_reset: 0xff,

            pin_d0: pin_d0.pin(),
            pin_d1: pin_d1.pin(),
            pin_d2: pin_d2.pin(),
            pin_d3: pin_d3.pin(),
            pin_d4: pin_d4.pin(),
            pin_d5: pin_d5.pin(),
            pin_d6: pin_d6.pin(),
            pin_d7: pin_d7.pin(),
            pin_vsync: pin_vsync.pin(),
            pin_href: pin_href.pin(),
            pin_pclk: pin_pclk.pin(),

            xclk_freq_hz: 20000000,
            ledc_timer: esp_idf_sys::ledc_timer_t_LEDC_TIMER_0,
            ledc_channel: esp_idf_sys::ledc_channel_t_LEDC_CHANNEL_0,

            pixel_format,
            frame_size,

            jpeg_quality: 12,
            fb_count: 1,
            grab_mode: camera::camera_grab_mode_t_CAMERA_GRAB_WHEN_EMPTY,

            fb_location: camera::camera_fb_location_t_CAMERA_FB_IN_PSRAM,

            __bindgen_anon_1: camera::camera_config_t__bindgen_ty_1 {
                pin_sccb_sda: pin_sda.pin(),
            },
            __bindgen_anon_2: camera::camera_config_t__bindgen_ty_2 {
                pin_sccb_scl: pin_scl.pin(),
            },

            ..Default::default()
        };

        esp_idf_sys::esp!(unsafe { camera::esp_camera_init(&config) })?;
        Ok(Self { _p: PhantomData })
    }

    pub fn get_framebuffer(&self) -> Option<FrameBuffer> {
        let fb = unsafe { camera::esp_camera_fb_get() };
        if fb.is_null() {
            //unsafe { camera::esp_camera_fb_return(fb); }
            None
        } else {
            Some(FrameBuffer {
                fb,
                _p: PhantomData,
            })
        }
    }

    pub fn sensor(&self) -> CameraSensor<'a> {
        CameraSensor {
            sensor: unsafe { camera::esp_camera_sensor_get() },
            _p: PhantomData,
        }
    }
}

impl<'a> Drop for Camera<'a> {
    fn drop(&mut self) {
        esp!(unsafe { camera::esp_camera_deinit() }).expect("error during esp_camera_deinit")
    }
}
