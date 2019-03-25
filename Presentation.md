# Presentation of the project: E-Toaster

## Introduction

We wanted to upgrade a toaster to be able to start it via a bluetooth module. The idea was that the mcu pilot a relay that will enable the start of the toaster. We also wanted to make it display information with the use of a screen. The information we wanted to display were the temperature (get by a sensor) or a message when the bread is ready. We also tried to implement a system of LEDs to see when the bluetooth module is working or the duration of the toasting.

## Choice of the components

The choice of the components was made following our given budget and needs. We tried to keep low prices as much as possible. We also selected means of communication that we already knew, like SPI and UART. One of the criteria was also that the product has a good datasheet, which simplify a lot of the work.

## Design of the Kicad

We started to design the implementation of the essentials components, including the microcontroller, the voltage regulator system and the programmable pins. Then we added our own components. Finally we added some more backup pins and some testpoints to check is everything is working as intended.

## Soldering

It was our first time trying to solder SMD components. We had to use our second board because of some problems on the first one (probably the MCU that died). We started by soldering the power system and checking if it was working as intented, to be sure to not damage anything again. Then everytime we soldered something, we used the testpoints to check if the connection was well made. The hardest thing to solder was the bluetooth module and it involved the use of the "SMD Rework Station".
At some point we had a problem with the usb connector. The print on our Kicad was reversed compared to the reality. It caused us to send 5V on the ground and vice-versa. So we had to use our backup pins to get the system working. 

## Use of the bluetooth module

We were able to communicate with the bluetooth module using a serial communication with a computer. we could modify some parameters of it or simply send data to the mcu with an UART connection. We also implemented two LEDs, one green and one red. The green LED tells us if the module wait for a connection, is connected or is in control mode. The red LED lights up everytime we send a data to the mcu.
