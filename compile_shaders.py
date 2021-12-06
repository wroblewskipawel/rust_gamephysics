import shutil
import subprocess
import os

if shutil.which("glslc"):
    source_dir = os.path.relpath("shaders/src/")
    spirv_dir = os.path.relpath("shaders/spv/")
    if os.path.isdir(spirv_dir):
        shutil.rmtree(spirv_dir)
    os.mkdir(spirv_dir)
    for fname in os.listdir(source_dir):
        source_file = os.path.join(source_dir, fname)
        output_file = os.path.join(
            spirv_dir, f"{os.path.splitext(fname)[1][1:]}.spv")
        out = subprocess.run(f"glslc {source_file} -o {output_file}",
                             shell=True, capture_output=True)
        if out.stdout:
            print(out.stdout)
        if out.stderr:
            print(out.stderr)
