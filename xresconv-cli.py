#!/usr/bin/env python
# -*- coding: utf-8 -*-

import glob
import io
import locale
import os
import platform
import re
import shutil
import string
import sys
import tempfile
# ==================================================================================
import threading
import xml.etree.ElementTree as ET
from multiprocessing import cpu_count
from argparse import ArgumentParser
from subprocess import PIPE, STDOUT, Popen

from print_color import cprintf_stderr, cprintf_stdout, print_style

console_encoding = sys.getfilesystemencoding()
java_encoding = 'utf-8'

if 'utf-8' != sys.getdefaultencoding().lower():
    try:
        sys.setdefaultencoding('utf-8')
    except Exception:
        reload(sys)
        sys.setdefaultencoding('utf-8')

xconv_split_by_spaces = re.compile('\\s+', re.IGNORECASE)

xconv_options = {
    'version': '1.3.1',
    'conv_list': None,
    'real_run': True,
    'args': {},
    'ext_args_l1': [],
    'ext_args_l2': [],
    'work_dir': '.',
    'xresloader_path': 'xresloader.jar',
    'item': [],
    'parallelism': int((cpu_count() - 1) / 2) + 1,
    'java_options': [],
    'default_scheme': {},
    'data_version': None,
    'output_matrix': {
        'file_path': None,
        'outputs': []
    }
}

# 默认双线程，实际测试过程中java的运行优化反而比多线程更能提升效率
if xconv_options['parallelism'] > 2:
    xconv_options['parallelism'] = 2

xconv_xml_global_nodes = []
xconv_xml_list_item_nodes = []

usage = '%(prog)s [options...] <convert list file> [-- [xresloader options...]]'
parser = ArgumentParser(usage=usage)

parser.add_argument(
    "-v",
    "--version",
    action="store_true",
    help="show version and exit",
    dest="version",
    default=False)
parser.add_argument(
    "-s",
    "--scheme-name",
    action="append",
    help="only convert schemes with name <scheme name>",
    metavar="<scheme>",
    dest="rule_schemes",
    default=[])
parser.add_argument(
    "-t",
    "--test",
    action="store_true",
    help="test run and show cmds",
    dest="test",
    default=False)
parser.add_argument(
    "-p",
    "--parallelism",
    action="store",
    help="set parallelism task number(default:" +
    str(xconv_options['parallelism']) + ')',
    metavar="<number>",
    dest="parallelism",
    type=int,
    default=xconv_options['parallelism'])
parser.add_argument(
    "-j",
    "--java-option",
    action="append",
    help="add java options to command(example: Xmx=2048m)",
    metavar="<java option>",
    dest="java_options",
    default=[])
parser.add_argument(
    "-a",
    "--data-version",
    action="store",
    help="set data version, if set it's will ignore the data_version option in convert list file",
    metavar="<version>",
    dest="data_version",
    default=None)

parser.add_argument(
    "convert_list_file",
    nargs='+',
    help="convert list file(xml) and options will be passed to xresloader.jar",
    metavar="<convert list file> [-- [xresloader options...]]",
    default=[])

options = parser.parse_args()

if options.version:
    print(xconv_options['version'])
    exit(0)


def print_help_msg(err_code):
    parser.print_help()
    exit(err_code)


if 0 == len(options.convert_list_file):
    print_help_msg(-1)

xconv_options['conv_list'] = options.convert_list_file.pop(0)
xconv_options['ext_args_l2'] = options.convert_list_file
xconv_options['data_version'] = options.data_version
# ========================================= 全局配置解析 =========================================
''' 读取xml文件 '''


def load_xml_file(file_path):
    try:
        xml_doc = ET.parse(file_path)
    except Exception as e:
        print(e)
        exit(-2)

    root_node = xml_doc.getroot()

    if root_node == None:
        print('[ERROR] root node not found in xml')
        print_help_msg(-3)

    # 枚举include文件
    include_nodes = root_node.findall("./include")
    if include_nodes and len(include_nodes) > 0:
        dir_prefix = os.path.dirname(file_path)
        for include_node in include_nodes:
            include_file_path = include_node.text
            if include_file_path and len(include_file_path) > 1:
                if include_file_path[0] != '/' and include_file_path[1] != ':':
                    include_file_path = os.path.join(dir_prefix,
                                                     include_file_path)
                load_xml_file(include_file_path)

    global_nodes = root_node.findall("./global")
    if global_nodes and len(global_nodes) > 0:
        for node in global_nodes:
            xconv_xml_global_nodes.append({
                "file_path": file_path,
                "node": node
            })

    list_item_nodes = root_node.findall("./list/item")
    if list_item_nodes and len(list_item_nodes) > 0:
        for node in list_item_nodes:
            xconv_xml_list_item_nodes.append({
                "file_path": file_path,
                "node": node
            })


load_xml_file(xconv_options['conv_list'])


# global配置解析/合并
def load_global_options(gns):
    for global_node in gns:
        for global_option in global_node['node']:
            tag_name = global_option.tag.lower()
            text_value = global_option.text
            if text_value:
                trip_value = text_value.strip()
            else:
                trip_value = None

            if not trip_value:
                continue

            if tag_name == 'work_dir':
                xconv_options['work_dir'] = text_value

            elif tag_name == 'xresloader_path':
                xconv_options['xresloader_path'] = text_value

            elif tag_name == 'proto':
                xconv_options['args']['-p'] = trip_value

            elif tag_name == 'output_type':
                if global_node['file_path'] != xconv_options['output_matrix']['file_path']:
                    xconv_options['output_matrix']['outputs'] = []
                    xconv_options['output_matrix']['file_path'] = global_node['file_path']
                output_rule = {
                    'type': trip_value, 
                    'rename': None,
                    'tags': set(),
                    'classes': set()
                }
                rename_rule = global_option.get('rename')
                if rename_rule and rename_rule.strip():
                    output_rule['rename'] = rename_rule
                tag_rule = global_option.get('tag')
                if tag_rule and tag_rule.strip():
                    output_rule['tags'] = set(filter(lambda x: x, xconv_split_by_spaces.split(tag_rule.strip())))
                class_rule = global_option.get('class')
                if class_rule and class_rule.strip():
                    output_rule['classes'] = set(filter(lambda x: x, xconv_split_by_spaces.split(class_rule.strip())))
                    
                xconv_options['output_matrix']['outputs'].append(output_rule)

            elif tag_name == 'proto_file':
                xconv_options['args']['-f'] = '"' + text_value + '"'

            elif tag_name == 'output_dir':
                xconv_options['args']['-o'] = '"' + text_value + '"'

            elif tag_name == 'data_src_dir':
                xconv_options['args']['-d'] = '"' + text_value + '"'
            elif tag_name == 'data_version':
                if xconv_options['data_version'] is None:
                    xconv_options['data_version'] = text_value

            elif tag_name == 'rename':
                xconv_options['args']['-n'] = '"' + trip_value + '"'

            elif tag_name == 'option':
                xconv_options['ext_args_l1'].append(trip_value)
            elif tag_name == 'java_option':
                xconv_options['java_options'].append(trip_value)
            elif tag_name == 'default_scheme':
                if 'name' in global_option.attrib:
                    scheme_key = global_option.attrib['name']
                    if scheme_key in xconv_options['default_scheme']:
                        xconv_options['default_scheme'][scheme_key].append(
                            trip_value)
                    else:
                        xconv_options['default_scheme'][
                            scheme_key] = [text_value]
            else:
                print('[ERROR] unknown global configure ' + tag_name)


if xconv_xml_global_nodes and len(xconv_xml_global_nodes) > 0:
    load_global_options(xconv_xml_global_nodes)

# ----------------------------------------- 全局配置解析 -----------------------------------------

conv_list_dir = os.path.dirname(xconv_options['conv_list'])
if conv_list_dir:
    os.chdir(conv_list_dir)
os.chdir(xconv_options['work_dir'])

conv_start_msg = ('[NOTICE] start to run conv cmds on dir: {0}' + os.linesep).format(os.getcwd())
if sys.version_info.major >= 3:
    cprintf_stdout([print_style.FC_YELLOW], conv_start_msg)
else:
    conv_compat_py2_write_buffer = False
    try:
        cprintf_stdout([print_style.FC_YELLOW], conv_start_msg.decode(console_encoding))
    except Exception as _e:
        conv_compat_py2_write_buffer = True
        cprintf_stdout([print_style.FC_YELLOW], conv_start_msg)

if not os.path.exists(xconv_options['xresloader_path']):
    cprintf_stderr([print_style.FC_RED],
                   '[ERROR] xresloader not found.({0}, you can download it from https://github.com/xresloader/xresloader/releases)' + os.linesep,
                   xconv_options['xresloader_path'])
    exit(-4)


# ========================================= 转换表配置解析 =========================================
# 转换项配置解析/合并
def load_list_item_nodes(lis):
    for item_info in lis:
        item = item_info['node']
        conv_item_obj = {
            'file': False,
            'scheme': False,
            'options': [],
            'enable': False,
            'scheme_data': {},
            'tags': set(),
            'classes': set()
        }

        if 'file' in item.attrib:
            conv_item_obj['file'] = item.attrib['file']
        if 'scheme' in item.attrib:
            conv_item_obj['scheme'] = item.attrib['scheme']
        if 'tag' in item.attrib:
            conv_item_obj['tags'] = set(filter(lambda x: x, xconv_split_by_spaces.split(item.attrib['tag'])))
        if 'class' in item.attrib:
            conv_item_obj['classes'] = set(filter(lambda x: x, xconv_split_by_spaces.split(item.attrib['class'])))

        # 局部选项
        for local_option in item.findall('./option'):
            text_value = local_option.text
            if text_value:
                trip_value = text_value.strip()
            else:
                trip_value = None

            if not trip_value:
                continue

            conv_item_obj['options'].append(trip_value)

        # 局部选项
        for local_option in item.findall('./scheme'):
            text_value = local_option.text
            if text_value:
                trip_value = text_value.strip()
            else:
                trip_value = None

            if not trip_value:
                continue

            if 'name' in local_option.attrib:
                scheme_key = local_option.attrib['name']
                if scheme_key and scheme_key in conv_item_obj['scheme_data']:
                    conv_item_obj['scheme_data'][scheme_key].append(text_value)
                else:
                    conv_item_obj['scheme_data'][scheme_key] = [text_value]
        for key in xconv_options['default_scheme']:
            if key not in conv_item_obj['scheme_data']:
                conv_item_obj['scheme_data'][key] = xconv_options[
                    'default_scheme'][key]

        # 转换规则
        if not options.rule_schemes or 0 == len(
                options.rule_schemes) or conv_item_obj[
                    'scheme'] in options.rule_schemes:
            conv_item_obj['enable'] = True

        xconv_options['item'].append(conv_item_obj)


if xconv_xml_list_item_nodes and len(xconv_xml_list_item_nodes) > 0:
    load_list_item_nodes(xconv_xml_list_item_nodes)
# ----------------------------------------- 转换配置解析 -----------------------------------------

# ========================================= 生成转换命令 =========================================
if not xconv_options['data_version'] is None:
    xconv_options['args']['-a'] = '"' + str(xconv_options['data_version']) + '"'

##### 全局命令和配置
global_cmd_args_map = xconv_options['args'].copy()
global_cmd_args_prefix_array = []
global_cmd_args_suffix_array = []

if len(xconv_options['ext_args_l1']) > 0:
    global_cmd_args_prefix_array.extend(xconv_options['ext_args_l1'])

##### 命令行参数
if len(xconv_options['ext_args_l2']) > 0:
    global_cmd_args_suffix_array.extend(xconv_options['ext_args_l2'])

cmd_list = []
for conv_item in xconv_options['item']:
    if not conv_item['enable']:
        continue

    item_output_matrix = xconv_options['output_matrix']['outputs']
    if not item_output_matrix:
        item_output_matrix = [{}]

    for item_output in item_output_matrix:
        item_cmd_args_array = []

        # merge global options
        item_cmd_args_map = global_cmd_args_map.copy()
        if 'type' in item_output and item_output['type']:
            item_cmd_args_map['-t'] = item_output['type']
        if 'rename' in item_output and item_output['rename']:
            item_cmd_args_map['-n'] = '"{0}"'.format(item_output['rename'])
        if 'tags' in item_output and item_output['tags']:
            check_limit = False
            for tag in item_output['tags']:
                if tag in conv_item['tags']:
                    check_limit = True
                    break
            if not check_limit:
                continue
        if 'classes' in item_output and item_output['classes']:
            check_limit = False
            for tag in item_output['classes']:
                if tag in conv_item['classes']:
                    check_limit = True
                    break
            if not check_limit:
                continue

        for key in item_cmd_args_map:
            item_cmd_args_array.append(key)
            item_cmd_args_array.append(item_cmd_args_map[key])

        # add item options
        item_cmd_args_array.extend(conv_item['options'])
        if len(conv_item['options']) > 0:
            item_cmd_args_array.extend(conv_item['options'])

        # add item scheme
        if conv_item['file'] and conv_item['scheme']:
            item_cmd_args_array.append('-s')
            item_cmd_args_array.append('"{:s}"'.format(conv_item['file']))
            item_cmd_args_array.append('-m')
            item_cmd_args_array.append('"{:s}"'.format(conv_item['scheme']))
        else:
            for key in conv_item['scheme_data']:
                for opt_val in conv_item['scheme_data'][key]:
                    item_cmd_args_array.append('-m')
                    item_cmd_args_array.append('"{:s}={:s}"'.format(key, opt_val))
        
        item_cmd_args_array.extend(global_cmd_args_suffix_array)
        cmd_list.append(item_cmd_args_array)

cmd_list.reverse()
# ----------------------------------------- 生成转换命令 -----------------------------------------

exit_code = 0
all_worker_thread = []
cmd_picker_lock = threading.Lock()

def print_buffer_to_fd(fd, buffer):
    if sys.version_info.major >= 3:
        fd.write(buffer.decode(java_encoding))
    else:
        if console_encoding == java_encoding or conv_compat_py2_write_buffer:
            sys.stderr.write(buffer)
        else:
            sys.stderr.write(buffer.decode(java_encoding))

def print_stdout_func(pexec):
    for output_line in pexec.stdout.readlines():
        print_buffer_to_fd(sys.stdout, output_line)

def print_stderr_func(pexec):
    for output_line in pexec.stdout.readlines():
        print_buffer_to_fd(sys.stderr, output_line)

def worker_func(idx):
    global exit_code
    java_options = ['java']
    if len(options.java_options) > 0:
        for java_option in options.java_options:
            java_options.append('-{0}'.format(java_option))
    if len(xconv_options['java_options']) > 0:
        for java_option in xconv_options['java_options']:
            java_options.append(java_option)

    java_options.append("-Dfile.encoding={0}".format(java_encoding))
    java_options.append('-jar')
    java_options.append(xconv_options['xresloader_path'])
    java_options.append('--stdin')

    once_pick_count = len(xconv_options['output_matrix']['outputs'])
    if once_pick_count <= 1:
        once_pick_count = 1
    pexec = None
    if not options.test:
        pexec = Popen(java_options,
            stdin=PIPE,
            stdout=PIPE,
            stderr=PIPE,
            shell=False)

        worker_thd_print_stdout = threading.Thread(target=print_stdout_func, args=[pexec])
        worker_thd_print_stderr = threading.Thread(target=print_stderr_func, args=[pexec])
        worker_thd_print_stdout.start()
        worker_thd_print_stderr.start()

        while True:
            cmd_picker_lock.acquire()
            if len(cmd_list) <= 0:
                cmd_picker_lock.release()
                break

            for _ in range(0, once_pick_count):
                if not cmd_list:
                    break
                pexec.stdin.write(' '.join(cmd_list.pop()).encode(java_encoding))
                pexec.stdin.write(os.linesep.encode(java_encoding))

            cmd_picker_lock.release()
            pexec.stdin.flush()
        pexec.stdin.close()
        for output_line in pexec.stdout.readlines():
            print(output_line.decode(java_encoding))
        cmd_exit_code = pexec.wait()
        
        worker_thd_print_stdout.join()
        worker_thd_print_stderr.join()

        exit_code = exit_code + cmd_exit_code
    else:
        this_thd_cmds = []
        while True:
            cmd_picker_lock.acquire()
            if len(cmd_list) <= 0:
                cmd_picker_lock.release()
                break

            for _ in range(0, once_pick_count):
                if not cmd_list:
                    break

                # python2 must use encode string to bytes or there will be messy code
                # python3 must not use encode methed because it will transform string to bytes
                if sys.version_info.major < 3 and not conv_compat_py2_write_buffer:
                    this_thd_cmds.append(' '.join(cmd_list.pop()).encode(console_encoding))
                else:
                    this_thd_cmds.append(' '.join(cmd_list.pop()))
            cmd_picker_lock.release()

        cprintf_stdout([print_style.FC_GREEN], (
            '"{0}"' + os.linesep + '\t>{1}' + os.linesep
        ).format('" "'.join(java_options), (os.linesep + '\t>').join(this_thd_cmds)))


for i in range(0, options.parallelism):
    this_worker_thd = threading.Thread(target=worker_func, args=[i])
    this_worker_thd.start()
    all_worker_thread.append(this_worker_thd)

# 等待退出
for thd in all_worker_thread:
    thd.join()

# ----------------------------------------- 实际开始转换 -----------------------------------------

cprintf_stdout([print_style.FC_MAGENTA],
               '[INFO] all jobs done. {0} job(s) failed.{1}'.format(
                   exit_code, os.linesep))

exit(exit_code)
