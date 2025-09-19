#![no_std]
#![no_main]

use aya_ebpf::{bindings::xdp_action, macros::xdp, programs::XdpContext};
use aya_ebpf::helpers::generated::{bpf_l3_csum_replace, bpf_l4_csum_replace};
use aya_log_ebpf::*;

use core::mem;

mod bindings;

use bindings::{ethhdr, iphdr, udphdr};

const IPPROTO_UDP: u8 = 0x0011;

const ETH_P_IP: u16 = 0x0800;

const ETH_HDR_LEN: usize = mem::size_of::<ethhdr>();

const IP_HDR_LEN: usize = mem::size_of::<iphdr>();

#[inline(always)]

fn ptr_at<T>(ctx: &XdpContext, offset: usize) -> Option<*const T> {
    let start = ctx.data();
    let end = ctx.data_end();
    let len = mem::size_of::<T>();

    if start + offset + len > end {
        return None;
    }

    Some((start + offset) as *const T)
}


#[inline(always)]

fn ptr_at_mut<T>(ctx: &XdpContext, offset: usize) -> Option<*mut T> {
    let ptr = ptr_at::<T>(ctx, offset)?;

    Some(ptr as *mut T)
}

#[xdp]
pub fn xdp_udp_server(ctx: XdpContext) -> u32 {
    match try_xdp_udp_server(ctx) {
        Ok(ret) => ret,
        Err(_) => xdp_action::XDP_ABORTED,
    }
}

fn try_xdp_udp_server(ctx: XdpContext) -> Result<u32, u32> {
    info!(&ctx, "received a packet");

    let eth = ptr_at::<ethhdr>(&ctx, 0).ok_or(xdp_action::XDP_PASS)?;

    if unsafe { u16::from_be((*eth).h_proto) } != ETH_P_IP {
        return Ok(xdp_action::XDP_PASS);
    }

    let ip = ptr_at_mut::<iphdr>(&ctx, ETH_HDR_LEN).ok_or(xdp_action::XDP_PASS)?;

    if unsafe { (*ip).protocol } != IPPROTO_UDP {
        return Ok(xdp_action::XDP_PASS);
    }

    let udp = ptr_at_mut::<udphdr>(&ctx, ETH_HDR_LEN + IP_HDR_LEN).ok_or(xdp_action::XDP_PASS)?;

    let sport = unsafe { (*udp).source };

    let dport = unsafe { (*udp).dest };

    let saddr = unsafe { (*ip).__bindgen_anon_1.addrs.saddr };

    let daddr = unsafe { (*ip).__bindgen_anon_1.addrs.daddr };

    let dport_x = u16::from_be(dport);


    if dport_x != 9191 {
        return Ok(xdp_action::XDP_PASS);
    }

    // error!(&ctx, "received a packet");

    // switch source and destination
    unsafe {
        (*ip).__bindgen_anon_1.addrs.saddr = daddr;
        (*ip).__bindgen_anon_1.addrs.daddr = saddr;

        (*udp).source = dport;
        (*udp).dest = sport;
    }

    // we have UDP, return same content, it's an echo server
    Ok(xdp_action::XDP_TX)
}

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[unsafe(link_section = "license")]
#[unsafe(no_mangle)]
static LICENSE: [u8; 13] = *b"Dual MIT/GPL\0";
