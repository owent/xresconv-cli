#!/usr/bin/env python
# -*- coding: utf-8 -*-

import sys
import os, platform, locale
import shutil, re, string
import xml.etree.ElementTree as ET
import glob, getopt
from print_color import print_style, cprintf_stdout, cprintf_stderr

console_encoding = sys.getfilesystemencoding()

if 'utf-8' != sys.getdefaultencoding().lower():
    try:
        sys.setdefaultencoding('utf-8')
    except Exception:
        reload(sys)
        sys.setdefaultencoding('utf-8')

xconv_options = {
    'version': '1.0.0.0',
    'conv_list' : None,
    'real_run': True,
    'args' : {},
    'ext_args_l1': [],
    'ext_args_l2': [],
    'work_dir': '.',
    'xresloader_path': 'xresloader.jar',

    'rules': {
        'schemes': None
    },

    'item': []
}

def print_help_msg(err_code):
    print('usage: ' + sys.argv[0] + ' [options] <convert list file> [xresloader options...]')
    print('options:')
    print('-h, --help                       help messages')
    print('-s, --scheme-name <scheme name>  only convert schemes with name <scheme name>')
    print('-v, --version                    show version and exit')
    print('-t, --test                       test run and show cmds')
    exit(err_code)

opts, left_args = getopt.getopt(sys.argv[1:], 'hs:tv', ['help', 'version', 'scheme-name=', 'test'])
for opt_key, opt_val in opts:
    if opt_key in ('-h', '--help'):
        print_help_msg(0)
    elif opt_key in ('-v', '--version'):
        print(xconv_options['version'])
        exit(0)
    elif opt_key in ('-s', '--scheme-name'):
        if xconv_options['rules']['schemes'] is None:
            xconv_options['rules']['schemes'] = {}
        xconv_options['rules']['schemes'][opt_val] = True
    elif opt_key in ('-t', '--test'):
        xconv_options['real_run'] = False
    else:
        print_help_msg(0)

if 0 == len(left_args):
    print_help_msg(-1)

xconv_options['conv_list'] = left_args.pop(0)
xconv_options['ext_args_l2'] = left_args

try:
    xml_doc = ET.parse(xconv_options['conv_list'])
except Exception as e:
    print(e)
    exit(-2)

root_node = xml_doc.getroot()

if root_node == None:
    print('[ERROR] root node not found in xml')
    print_help_msg(-3)

# ========================================= 全局配置解析 =========================================
global_nodes = root_node.findall("./global")

if global_nodes and len(global_nodes) > 0:
    for global_node in global_nodes:
        for global_option in global_node:
            tag_name = global_option.tag.lower()
            text_value = global_option.text
            if text_value:
                trip_value = text_value.strip()
            else:
                trip_value = None

            if not trip_value:
                continue

            if 'work_dir' == tag_name:
                xconv_options['work_dir'] = text_value

            elif 'xresloader_path' == tag_name:
                xconv_options['xresloader_path'] = text_value

            elif 'proto' == tag_name:
                xconv_options['args']['-p'] = trip_value

            elif 'output_type' == tag_name:
                xconv_options['args']['-t'] = trip_value

            elif 'proto_file' == tag_name:
                xconv_options['args']['-f'] = '"' + text_value + '"'

            elif 'output_dir' == tag_name:
                xconv_options['args']['-o'] = '"' + text_value + '"'

            elif 'data_src_dir' == tag_name:
                xconv_options['args']['-d'] = '"' + text_value + '"'

            elif 'rename' == tag_name:
                xconv_options['args']['-n'] = '"' + trip_value + '"'

            elif 'option' == tag_name:
                xconv_options['ext_args_l1'].append(trip_value)

            else:
                print('[ERROR] unknown global configure ' + tag_name)

# ----------------------------------------- 全局配置解析 -----------------------------------------

conv_list_dir = os.path.dirname(xconv_options['conv_list'])
os.chdir(conv_list_dir)
os.chdir(xconv_options['work_dir'])

cprintf_stdout([print_style.FC_YELLOW], '[NOTICE] start to run conv cmds on dir: {0}\n', os.getcwd())

if not os.path.exists(xconv_options['xresloader_path']):
    cprintf_stderr([print_style.FC_RED], '[ERROR] xresloader not found.({0})\n', xconv_options['xresloader_path'])
    exit(-4)

# ========================================= 转换表配置解析 =========================================

for item in root_node.findall("./list/item"):
    conv_item_obj = {
        'file': item.attrib['file'],
        'scheme': item.attrib['scheme'],
        'options': [],
        'enable': False
    }

    # 局部选项
    for local_option in item.findall('./option'):
        tag_name = local_option.tag.lower()
        text_value = local_option.text
        if text_value:
            trip_value = text_value.strip()
        else:
            trip_value = None

        if not trip_value:
            continue

        if 'option' == tag_name:
            conv_item_obj['options'].append(trip_value)

    # 转换规则
    if xconv_options['rules']['schemes'] is None or conv_item_obj['scheme'] in xconv_options['rules']['schemes']:
        conv_item_obj['enable'] = True

    xconv_options['item'].append(conv_item_obj)
# ----------------------------------------- 转换配置解析 -----------------------------------------


# ========================================= 实际开始转换 =========================================
##### 全局命令和配置
global_cmd_prefix = 'java -jar "{0}"'.format(xconv_options['xresloader_path'])
for global_optk in xconv_options['args']:
    global_optv= xconv_options['args'][global_optk]
    global_cmd_prefix += ' ' + global_optk + ' ' + global_optv

if len(xconv_options['ext_args_l1']) > 0:
    global_cmd_prefix += ' ' + ' '.join(xconv_options['ext_args_l1'])

##### 命令行参数
global_cmd_suffix = ''
if len(xconv_options['ext_args_l2']) > 0:
    global_cmd_suffix += ' ' + ' '.join(xconv_options['ext_args_l2'])


for conv_item in xconv_options['item']:
    if not conv_item['enable']:
        continue

    item_cmd_options = ''
    if len(conv_item['options']) > 0:
        item_cmd_options += ' ' + ' '.join(conv_item['options'])

    cmd_scheme_info = ' -s "{:s}" -m "{:s}"'.format(conv_item['file'], conv_item['scheme'])
    run_cmd = global_cmd_prefix + item_cmd_options + cmd_scheme_info + global_cmd_suffix
    if 'utf-8' != console_encoding.lower():
        run_cmd = run_cmd.encode(console_encoding)
    cprintf_stdout([print_style.FC_GREEN], '[INFO] {0}\n', run_cmd)
    if xconv_options['real_run']:
        os.system(run_cmd)
# ----------------------------------------- 实际开始转换 -----------------------------------------

cprintf_stdout([print_style.FC_MAGENTA], '[INFO] all jobs done.\n')