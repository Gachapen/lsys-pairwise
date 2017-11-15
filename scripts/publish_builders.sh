#!/bin/bash

source ./vars.sh

sudo docker push $front_builder:latest
sudo docker push $back_builder:latest
