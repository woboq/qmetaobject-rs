/****************************************************************************
**
** Copyright (C) 2017 The Qt Company Ltd.
** Contact: https://www.qt.io/licensing/
**
** This file is part of the examples of the Qt Toolkit.
**
** $QT_BEGIN_LICENSE:BSD$
** Commercial License Usage
** Licensees holding valid commercial Qt licenses may use this file in
** accordance with the commercial license agreement provided with the
** Software or, alternatively, in accordance with the terms contained in
** a written agreement between you and The Qt Company. For licensing terms
** and conditions see https://www.qt.io/terms-conditions. For further
** information use the contact form at https://www.qt.io/contact-us.
**
** BSD License Usage
** Alternatively, you may use this file under the terms of the BSD license
** as follows:
**
** "Redistribution and use in source and binary forms, with or without
** modification, are permitted provided that the following conditions are
** met:
**   * Redistributions of source code must retain the above copyright
**     notice, this list of conditions and the following disclaimer.
**   * Redistributions in binary form must reproduce the above copyright
**     notice, this list of conditions and the following disclaimer in
**     the documentation and/or other materials provided with the
**     distribution.
**   * Neither the name of The Qt Company Ltd nor the names of its
**     contributors may be used to endorse or promote products derived
**     from this software without specific prior written permission.
**
**
** THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
** "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
** LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
** A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
** OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
** SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
** LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
** DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
** THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
** (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
** OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE."
**
** $QT_END_LICENSE$
**
****************************************************************************/

#include "noisynode.h"

#include <QtCore/QRandomGenerator>
#include <QtQuick/QSGSimpleMaterialShader>
#include <QtQuick/QSGTexture>
#include <QtQuick/QQuickWindow>

#define NOISE_SIZE 64

struct NoisyMaterial
{
    ~NoisyMaterial() {
        delete texture;
    }

    QColor color;
    QSGTexture *texture;
};

class NoisyShader : public QSGSimpleMaterialShader<NoisyMaterial>
{
    QSG_DECLARE_SIMPLE_SHADER(NoisyShader, NoisyMaterial)

public:
    NoisyShader() : id_color(-1), id_texture(-1), id_textureSize(-1) {
        setShaderSourceFile(QOpenGLShader::Vertex, ":/scenegraph/graph/shaders/noisy.vsh");
        setShaderSourceFile(QOpenGLShader::Fragment, ":/scenegraph/graph/shaders/noisy.fsh");
    }

    QList<QByteArray> attributes() const {  return QList<QByteArray>() << "aVertex" << "aTexCoord"; }

    void updateState(const NoisyMaterial *m, const NoisyMaterial *) {

        // Set the color
        program()->setUniformValue(id_color, m->color);

        // Bind the texture and set program to use texture unit 0 (the default)
        m->texture->bind();

        // Then set the texture size so we can adjust the texture coordinates accordingly in the
        // vertex shader..
        QSize s = m->texture->textureSize();
        program()->setUniformValue(id_textureSize, QSizeF(1.0 / s.width(), 1.0 / s.height()));
    }

    void resolveUniforms() {
        id_texture = program()->uniformLocation("texture");
        id_textureSize = program()->uniformLocation("textureSize");
        id_color = program()->uniformLocation("color");

        // We will only use texture unit 0, so set it only once.
        program()->setUniformValue(id_texture, 0);
    }

private:
    int id_color;
    int id_texture;
    int id_textureSize;
};

NoisyNode::NoisyNode(QQuickWindow *window)
{
    // Make some noise...
    QImage image(NOISE_SIZE, NOISE_SIZE, QImage::Format_RGB32);
    uint *data = (uint *) image.bits();
    for (int i=0; i<NOISE_SIZE * NOISE_SIZE; ++i) {
        uint g = QRandomGenerator::global()->bounded(0xff);
        data[i] = 0xff000000 | (g << 16) | (g << 8) | g;
    }

    QSGTexture *t = window->createTextureFromImage(image);
    t->setFiltering(QSGTexture::Nearest);
    t->setHorizontalWrapMode(QSGTexture::Repeat);
    t->setVerticalWrapMode(QSGTexture::Repeat);

    QSGSimpleMaterial<NoisyMaterial> *m = NoisyShader::createMaterial();
    m->state()->texture = t;
    m->state()->color = QColor::fromRgbF(0.95, 0.95, 0.97);
    m->setFlag(QSGMaterial::Blending);

    setMaterial(m);
    setFlag(OwnsMaterial, true);

    QSGGeometry *g = new QSGGeometry(QSGGeometry::defaultAttributes_TexturedPoint2D(), 4);
    QSGGeometry::updateTexturedRectGeometry(g, QRect(), QRect());
    setGeometry(g);
    setFlag(OwnsGeometry, true);
}

void NoisyNode::setRect(const QRectF &bounds)
{
    QSGGeometry::updateTexturedRectGeometry(geometry(), bounds, QRectF(0, 0, 1, 1));
    markDirty(QSGNode::DirtyGeometry);
}
