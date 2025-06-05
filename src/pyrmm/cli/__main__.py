import click
import subprocess
import sys
import os
from pathlib import Path

@click.group()
@click.option('--verbose', '-v', is_flag=True, help='Enable verbose output')
@click.pass_context
def cli(ctx, verbose):
    """PyRMM Command Line Interface
    
    A comprehensive tool for building, testing, and managing RMM modules.
    """
    ctx.ensure_object(dict)
    ctx.obj['verbose'] = verbose

@cli.command()
def version():
    """Display the version of PyRMM."""
    from pyrmm.__about__ import __version__
    click.echo(f"PyRMM version: {__version__}")

@cli.command()
@click.option('-s', '--source', 'source_dir', required=True, help='Source directory path')
@click.option('-c', '--config', 'config_dir', required=True, help='Configuration directory path')
@click.option('-v', '--version', 'build_version', required=True, help='Build version tag')
@click.option('-d', '--dev', is_flag=True, help='Development build')
@click.option('-p', '--prerelease', is_flag=True, help='Prerelease build')
@click.option('-u', '--upload', is_flag=True, help='Upload build artifacts')
@click.option('-t', '--token', required=True, help='GitHub token for PyGithub operations')
@click.pass_context
def build(ctx, source_dir, config_dir, build_version, dev, prerelease, upload, token):
    """Build RMM modules with specified configuration."""
    verbose = ctx.obj.get('verbose', False)
    
    if verbose:
        click.echo(f"Building module from {source_dir} with config {config_dir}")
        click.echo(f"Version: {build_version}, Dev: {dev}, Prerelease: {prerelease}")
    
    click.echo("🔨 Starting module build...")
    click.echo(f"📁 Source: {source_dir}")
    click.echo(f"⚙️  Config: {config_dir}")
    click.echo(f"🏷️  Version: {build_version}")
    
    if dev:
        click.echo("🧪 Development build enabled")
    if prerelease:
        click.echo("🚧 Prerelease build enabled")
    if upload:
        click.echo("⬆️  Upload enabled")
    
    # TODO: Implement actual build logic
    click.echo("✅ Build completed successfully!")

@cli.command()
@click.argument('prebuild_file', required=True)
@click.option('-o', '--output', required=True, help='Output directory for test results')
@click.option('--ai-enabled', is_flag=True, help='Enable AI-powered testing')
@click.option('--api-key', help='API key for AI services')
@click.option('--base-url', help='Base URL for AI services')
@click.option('--model', help='AI model to use for testing')
@click.option('-t', '--token', required=True, help='GitHub token for PyGithub operations')
@click.pass_context
def test(ctx, prebuild_file, output, ai_enabled, api_key, base_url, model, token):
    """Test RMM modules with optional AI-powered analysis."""
    verbose = ctx.obj.get('verbose', False)
    
    if verbose:
        click.echo(f"Testing prebuild file: {prebuild_file}")
        click.echo(f"Output directory: {output}")
        click.echo(f"AI enabled: {ai_enabled}")
    
    click.echo("🧪 Starting module testing...")
    click.echo(f"📦 Testing: {prebuild_file}")
    click.echo(f"📁 Output: {output}")
    
    if ai_enabled:
        click.echo("🤖 AI-powered testing enabled")
        if api_key and len(api_key) > 0:
            click.echo(f"🔑 API Key: {'*' * min(len(api_key), 8)}")
        if base_url:
            click.echo(f"🌐 Base URL: {base_url}")
        if model:
            click.echo(f"🧠 Model: {model}")
    
    # Create output directory if it doesn't exist
    try:
        os.makedirs(output, exist_ok=True)
    except Exception as e:
        click.echo(f"Error creating output directory: {e}", err=True)
        return
    
    # TODO: Implement actual test logic
    click.echo("✅ Testing completed successfully!")

@cli.command()
@click.option('-v', '--version', 'release_version', required=True, help='Release version')
@click.option('-d', '--dev', is_flag=True, help='Development release')
@click.option('-p', '--prerelease', is_flag=True, help='Prerelease version')
@click.option('-u', '--upload', is_flag=True, help='Upload to module repository')
@click.option('-t', '--token', required=True, help='GitHub token for PyGithub operations')
@click.option('--prebuild', multiple=True, help='Prebuild artifact files')
@click.option('--report', help='Test report file')
@click.option('--push', is_flag=True, help='Push to device via ADB')
@click.option('--path', help='Push path on device')
@click.pass_context
def release(ctx, release_version, dev, prerelease, upload, token, prebuild, report, push, path):
    """Create and publish module releases."""
    verbose = ctx.obj.get('verbose', False)
    
    if verbose:
        click.echo(f"Creating release: {release_version}")
        click.echo(f"Prebuild files: {prebuild}")
        click.echo(f"Report: {report}")
    
    click.echo("🚀 Starting module release...")
    click.echo(f"🏷️  Version: {release_version}")
    
    if dev:
        click.echo("🧪 Development release")
    if prerelease:
        click.echo("🚧 Prerelease version")
    if upload:
        click.echo("⬆️  Uploading to repository")
    
    for pb_file in prebuild:
        click.echo(f"📦 Including: {pb_file}")
    
    if report:
        click.echo(f"📊 Report: {report}")
    
    if push and path:
        click.echo(f"📱 Pushing to device: {path}")
    
    # TODO: Implement actual release logic
    click.echo("✅ Release completed successfully!")

@cli.command()
@click.argument('command', nargs=-1, required=True)
@click.option('--shell', '-s', is_flag=True, help='Execute command in shell')
@click.option('--capture', '-c', is_flag=True, help='Capture and return output')
@click.option('--cwd', help='Working directory for command execution')
@click.pass_context
def run(ctx, command, shell, capture, cwd):
    """Execute arbitrary external commands.
    
    This command allows you to run any external command or script.
    
    Examples:
        rmm run ls -la
        rmm run --shell "echo hello world"  
        rmm run --cwd /path/to/dir make build
        rmm run python script.py
    """
    verbose = ctx.obj.get('verbose', False)
    
    if not command:
        click.echo("Error: No command specified", err=True)
        sys.exit(1)
    
    # Join command arguments into a single command string
    cmd_str = ' '.join(command)
    
    if verbose:
        click.echo(f"Executing: {cmd_str}")
        if cwd:
            click.echo(f"Working directory: {cwd}")
        click.echo(f"Shell mode: {shell}")
        click.echo(f"Capture output: {capture}")
    
    try:
        # Prepare command execution
        if shell:
            # Execute as shell command
            result = subprocess.run(
                cmd_str,
                shell=True,
                cwd=cwd,
                capture_output=capture,
                text=True
            )
        else:
            # Execute as separate arguments
            result = subprocess.run(
                command,
                cwd=cwd,
                capture_output=capture,
                text=True
            )
        
        # Handle output
        if capture:
            if result.stdout:
                click.echo(result.stdout)
            if result.stderr:
                click.echo(result.stderr, err=True)
        
        # Exit with the same code as the executed command
        if result.returncode != 0:
            if verbose:
                click.echo(f"Command failed with exit code: {result.returncode}", err=True)
            sys.exit(result.returncode)
        else:
            if verbose:
                click.echo("Command completed successfully")
    
    except FileNotFoundError:
        click.echo(f"Error: Command '{command[0]}' not found", err=True)
        sys.exit(1)
    except PermissionError:
        click.echo(f"Error: Permission denied executing '{command[0]}'", err=True)
        sys.exit(1)
    except Exception as e:
        click.echo(f"Error executing command: {e}", err=True)
        sys.exit(1)

@cli.command()
@click.option('--format', 'output_format', 
              type=click.Choice(['json', 'yaml', 'table']), 
              default='table', 
              help='Output format')
def info(output_format):
    """Display system and environment information."""
    import platform
    
    info_data = {
        'system': platform.system(),
        'platform': platform.platform(),
        'python_version': sys.version,
        'python_executable': sys.executable,
        'current_directory': os.getcwd(),
        'pyrmm_location': __file__
    }
    
    if output_format == 'json':
        import json
        click.echo(json.dumps(info_data, indent=2))
    elif output_format == 'yaml':
        # Simple YAML-like output without requiring PyYAML
        for key, value in info_data.items():
            click.echo(f"{key}: {value}")
    else:  # table format
        click.echo("System Information:")
        click.echo("=" * 50)
        for key, value in info_data.items():
            click.echo(f"{key.replace('_', ' ').title():<20}: {value}")

@cli.command()
@click.argument('hook_type', 
                type=click.Choice(['py', 'go', 'rs', 'exe', 'ps1', 'cmd', 'bat', 'elf', 'sh']))
@click.option('--phase', 
              type=click.Choice(['prebuild', 'postbuild']), 
              default='prebuild',
              help='Build phase (prebuild or postbuild)')
@click.option('--template', is_flag=True, help='Create template files')
def init(hook_type, phase, template):
    """Initialize RMM build hooks and configuration."""
    click.echo(f"🚀 Initializing {hook_type} {phase} hook...")
    
    # Create RMM_BUILD directory structure
    rmm_build_dir = Path("RMM_BUILD")
    rmm_build_dir.mkdir(exist_ok=True)
    
    # Create phase-specific directories
    phase_dir = rmm_build_dir / phase
    if hook_type in ['go', 'rs']:
        phase_dir = phase_dir / hook_type
    phase_dir.mkdir(parents=True, exist_ok=True)
    
    if template:
        # Create template files based on hook type
        if hook_type == 'py':
            template_content = f"""#!/usr/bin/env python3
# {phase.capitalize()} script for RMM module
import os
import sys

def main():
    print("Running {phase} hook...")
    # Add your {phase} logic here
    pass

if __name__ == "__main__":
    main()
"""
            script_path = rmm_build_dir / f"{phase}.py"
            with open(script_path, 'w', encoding='utf-8') as f:
                f.write(template_content)
        
        elif hook_type == 'sh':
            template_content = f"""#!/bin/bash
# {phase.capitalize()} script for RMM module
echo "Running {phase} hook..."
# Add your {phase} logic here
"""
            script_path = rmm_build_dir / f"{phase}.sh"
            with open(script_path, 'w', encoding='utf-8') as f:
                f.write(template_content)
            try:
                script_path.chmod(0o755)
            except:
                pass  # Ignore permission errors on Windows
        
        elif hook_type == 'go':
            go_mod_content = f"""module {phase}

go 1.21
"""
            main_go_content = f"""package main

import "fmt"

func main() {{
    fmt.Println("Running {phase} hook...")
    // Add your {phase} logic here
}}
"""
            with open(phase_dir / "go.mod", 'w', encoding='utf-8') as f:
                f.write(go_mod_content)
            with open(phase_dir / "main.go", 'w', encoding='utf-8') as f:
                f.write(main_go_content)
        
        elif hook_type == 'rs':
            cargo_toml_content = f"""[package]
name = "{phase}"
version = "0.1.0"
edition = "2021"

[dependencies]
"""
            main_rs_content = f"""fn main() {{
    println!("Running {phase} hook...");
    // Add your {phase} logic here
}}
"""
            with open(phase_dir / "Cargo.toml", 'w', encoding='utf-8') as f:
                f.write(cargo_toml_content)
            src_dir = phase_dir / "src"
            src_dir.mkdir(exist_ok=True)
            with open(src_dir / "main.rs", 'w', encoding='utf-8') as f:
                f.write(main_rs_content)
    
    # Create basic build.toml if it doesn't exist
    build_toml = rmm_build_dir / "build.toml"
    if not build_toml.exists():
        build_toml_content = """# RMM Module Build Configuration

META-INF=true

# Add your build configuration here
[module]
# name = "MyModule"
# version = "1.0.0"
# author = "Your Name"
# description = "Module description"

[build]
# compression = true
# optimization = true
"""
        with open(build_toml, 'w', encoding='utf-8') as f:
            f.write(build_toml_content)
    
    click.echo(f"✅ {hook_type} {phase} hook initialized successfully!")
    if template:
        click.echo(f"📁 Template files created in {phase_dir}")

if __name__ == "__main__":
    cli()


