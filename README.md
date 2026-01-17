Copyright (C) Joshua Smith & Rashid Byrdsell - All Rights Reserved
Unauthorized copying of this file, via any medium is strictly prohibited
Proprietary and confidential
Written by Joshua Smith <JoshuaEthanSmith@proton.me>, October 12, 2025

 # Features
 This software is intended to provide a user interface for controlling unmanned aerial vehicles with hand gestures. Its features include (but are not limited to): 
 * Vehicle following
 * User control of the vehicle via hand gesture
 * Vehicle collision avoidance
 * Vehicle photography

# Technologies
* TensorFlow for computer vision related tasks
  * Google's [MediaPipe](https://ai.google.dev/edge/mediapipe/solutions/vision/gesture_recognizer) for gesture recognition model

# Installation
This software includes all the necessary binaries to function as a standalone application, provided no alterations or modifications are made to the directories herein.

# Development Setup
Since the binaries are included, and to make this process as simple as possible, the only necessary steps to get set up for development are installing RustRover, Cargo, and LLVM, then setting the following environment variable
`TFLITEC_PREBUILT_PATH={...}\TakeFlight\3rd_party\TensorFlow\tensorflowlite_c.dll`
Once these steps are complete, the project's build system will take care of the rest.

# Contributors
Rashid Byrdsell

Joshua Smith

# Project Status
This project is in pre-alpha. The project is quite volatile, and missing implementation for certain core features.
