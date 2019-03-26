# Presentation of the project: E-Toaster

## Introduction

We wanted to upgrade a toaster to be able to start it via a bluetooth module. The idea was that the mcu pilot a relay that will enable the start of the toaster. We also wanted to make it display information with the use of a screen. The information we wanted to display were the temperature (get by a sensor) or a message when the bread is ready. We also tried to implement a system of LEDs to see when the bluetooth module or the toaster is working.

## Choice of the components

The choice of the components was made following our given budget and needs. We tried to keep low prices as much as possible. We also selected means of communication that we already knew, like SPI and UART. One of the criteria was also that the product has a good datasheet, which simplify a lot of the work.

## Design of the Kicad

We started to design the implementation of the essentials components, including the microcontroller, the voltage regulator system and the programmable pins. Then we added our own components. Finally we added some more backup pins and some testpoints to check is everything is working as intended.
We also added two buttons, one to reset the MCU and one to control the screen.

## Soldering

It was our first time trying to solder SMD components. We had to use our second board because of some problems on the first one (probably the MCU that died). We started by soldering the power system and checking if it was working as intented, to be sure to not damage anything again. Then everytime we soldered something, we used the testpoints to check if the connection was well made. The hardest thing to solder was the bluetooth module and it involved the use of the "SMD Rework Station".
At some point we had a problem with the usb connector. The print on our Kicad was reversed compared to the reality. It caused us to send 5V on the ground and vice-versa. So we had to use our backup pins to get the system working. 

## Use of the bluetooth module

We were able to communicate with the bluetooth module using a serial communication with a computer, only on Windows however. we could modify some parameters of it or simply send data to the MCU with an UART connection. We also implemented two LEDs, one green and one red. The green LED tells us if the module wait for a connection, is connected or is in control mode. The red LED lights up everytime we send a data to the MCU.


## Temperature sensor and ADC

Communicating with the temperature sensor was no problem and was done easily, what was more of a challenge was the ADC as the crate used didn't have the ADC component, this however turned out to be due to an outdated crate and updating it solved the issue. In the datasheet for the temperature sensor there was a formula for the conversion between millivolts and degrees Celcius

![Eq](https://raw.githubusercontent.com/oscrim/E7020E-Project/master/pictures/Millivolt2Celcius.png)

It was however not possible to do this kind of mathematics on the PCB since we are unable to use the standard libraries, therefore we had to approximate the conversion using a linear model instead which gave us.

![Est](https://raw.githubusercontent.com/oscrim/E7020E-Project/master/pictures/Estimate.png)

This is not perfect but it is close enough for the temperatures we have, we also decided that since some of the calculations will have too many decimals and we cant easily change the number of decimals we will round the temperatures to the closest integer
