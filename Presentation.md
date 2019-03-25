# Presentation of the project: E-Toaster

## Introduction

We wanted to upgrade a toaster to be able to start it via a bluetooth module. We also wanted to make it display information with the use of a screen. The information we wanted to display were the temperature (get by a sensor) or a message when the bread is ready. We also tried to implement a system of LEDs to see when the bluetooth module is working or the duration of the toasting.

## Choice of the components

The choice of the components was made following our given budget and needs. We tried to keep low prices as much as possible. We also selected means of communication that we already knew, like SPI and UART. One of the criteria was also that the product has a good datasheet, which simplify a lot of the work.

## Design of the Kicad

We started to design the implementation of the essentials components, including the microcontroller, the voltage regulator system and the programmable pins. Then we added our own components. Finally we added some more backup pins and some testpoints to check is everything is working as intended.

