## Description

- lib-gst-meet is server side rust implementation of (lib-jitsi-meet for browser ), it allows to record and stream jitsi meet conferences without using headless chrome to save cost and resouces, it intercepts RTP packets and write to gstreamer to sink pad.  

## Components 

 - Gstreamer
 - Redis
 - actix-web server
 - lib-gst-meet is rust implementation of lib-jitsi-meet javascript library for browser
 - autoscalable pipeline

## Deployment 
 - please refer to Makefile
 
