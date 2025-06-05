"""
Pyrmm Init Command group 

rmm init --magisk < 目标版本:latest > --id <模块ID> --name <模块名称> --author <作者> --description/-d <描述> --version/-v <版本号> 
         --apatch
         --kernelsu
         -m
         -a
         -k


"""
from .__main__ import init
__all__ = ['init']